use crate::{Block, If, Match, Session, Type, lower_hir_if};
use sodigy_error::{Error, ErrorKind, to_ordinal};
use sodigy_hir as hir;
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_parse::Field;
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::{InternedString, intern_string};
use sodigy_token::InfixOp;
use std::collections::hash_map::{Entry, HashMap};

mod dispatch;

#[derive(Clone, Debug)]
pub enum Expr {
    Ident(IdentWithOrigin),
    Number {
        n: InternedNumber,
        span: Span,
    },
    // Ideally, we can create `Callable::StringInit`, but that wouldn't work well with long strings.
    String {
        binary: bool,
        s: InternedString,
        span: Span,
    },
    Char {
        ch: u32,
        span: Span,
    },
    Byte {
        b: u8,
        span: Span,
    },
    If(If),

    // `Match` is later lowered to a `Block`.
    Match(Match),
    Block(Block),
    Path {
        lhs: Box<Expr>,
        fields: Vec<Field>,
    },
    FieldModifier {
        fields: Vec<(InternedString, Span)>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        func: Callable,
        args: Vec<Expr>,
        arg_group_span: Span,

        // If it's a generic function, def_spans of its generics (T, U, ...)
        // are stored here so that `inter_mir::TypeSolver::solve_expr` can use.
        generic_defs: Vec<Span>,

        // It helps generating error messages.
        // It has type `Vec<(keyword: InternedString, n: usize)>` where
        // nth argument in `args` has keyword `keyword`.
        given_keyword_arguments: Vec<(InternedString, usize)>,
    },
}

#[derive(Clone, Debug)]
pub enum Callable {
    // There must be `HashMap<Span, Func>` somewhere
    Static {
        def_span: Span,
        span: Span,
    },
    StructInit {
        def_span: Span,
        span: Span,
    },
    TupleInit {
        group_span: Span,
    },
    ListInit {
        group_span: Span,
    },
    // It's a functor and can only be evaluated at runtime.
    Dynamic(Box<Expr>),
}

#[derive(Clone, Copy, Debug)]
pub enum ShortCircuitKind {
    And,
    Or,
}

impl Expr {
    pub fn from_hir(hir_expr: &hir::Expr, session: &mut Session) -> Result<Expr, ()> {
        match hir_expr {
            hir::Expr::Ident(id) => Ok(Expr::Ident(*id)),
            hir::Expr::Number { n, span } => Ok(Expr::Number {
                n: n.clone(),
                span: *span,
            }),
            hir::Expr::String { binary, s, span } => Ok(Expr::String {
                binary: *binary,
                s: *s,
                span: *span,
            }),
            hir::Expr::Char { ch, span } => Ok(Expr::Char {
                ch: *ch,
                span: *span,
            }),
            hir::Expr::Byte { b, span } => Ok(Expr::Byte {
                b: *b,
                span: *span,
            }),

            // `if let` is `hir::If`, but is lowered to `mir::Match`.
            hir::Expr::If(r#if) => Ok(lower_hir_if(r#if, session)?),

            hir::Expr::Match(r#match) => Ok(Expr::Match(Match::from_hir(r#match, session)?)),
            hir::Expr::Block(block) => Ok(Expr::Block(Block::from_hir(block, session)?)),
            hir::Expr::Call {
                func,
                args: hir_args,
                arg_group_span,
            } => {
                let mut has_error = false;
                let mut def_span = None;
                let mut generic_defs = vec![];
                let mut given_keyword_arguments = vec![];

                let (call_span, func) = match Expr::from_hir(func, session) {
                    Ok(e @ Expr::Ident(id)) => match id.origin {
                        NameOrigin::Local { kind } |
                        NameOrigin::Foreign { kind } => match kind {
                            NameKind::Func => {
                                def_span = Some(id.def_span);
                                (
                                    id.span,
                                    Callable::Static {
                                        def_span: id.def_span,
                                        span: id.span,
                                    },
                                )
                            },
                            // The programmer defines a functor using `let` keyword
                            // and calls it. In this case, we have to dynamically call the
                            // function on runtime. (Maybe we can do some optimizations and turn it into a static call?)
                            NameKind::Let { .. } => {
                                def_span = Some(id.def_span);
                                (id.span, Callable::Dynamic(Box::new(e)))
                            },
                            _ => panic!("TODO: {kind:?}"),
                        },
                        NameOrigin::FuncParam { .. } => (id.span, Callable::Dynamic(Box::new(e))),
                        NameOrigin::Generic { .. } => unreachable!(),
                        NameOrigin::External => unreachable!(),
                    },
                    Ok(func) => (func.error_span_wide(), Callable::Dynamic(Box::new(func))),
                    Err(()) => {
                        has_error = true;
                        todo!()
                    },
                };

                // If we know `def_span` and the `def_span` is in `func_shapes`,
                // we know the exact definition of the function, and can process keyword arguments and default values.
                let mut mir_args = match def_span {
                    Some(def_span) => match session.func_shapes.get(&def_span) {
                        Some(func_shape) => {
                            for generic_def in func_shape.generics.iter() {
                                session.generic_instances.insert(
                                    (call_span, generic_def.name_span),
                                    Type::GenericInstance {
                                        call: call_span,
                                        generic: generic_def.name_span,
                                    },
                                );
                                generic_defs.push(generic_def.name_span);
                            }

                            let params = func_shape.params.to_vec();
                            let mut mir_args: Vec<Option<Expr>> = vec![None; params.len().max(hir_args.len())];

                            // used for error messages
                            let mut given_keyword_arguments_ = vec![None; params.len().max(hir_args.len())];

                            // Positional args cannot come after a keyword arg, and hir guarantees that.
                            let mut positional_arg_cursor = 0;

                            // Another attempt for even better error messages
                            let mut repeated_arguments: HashMap<InternedString, Vec<RenderableSpan>> = HashMap::new();

                            for hir_arg in hir_args.iter() {
                                match hir_arg.keyword {
                                    Some((keyword, keyword_span)) => {
                                        let mut arg_index = None;

                                        // It's O(n), but n is very small
                                        for (i, param) in params.iter().enumerate() {
                                            if param.name == keyword {
                                                arg_index = Some(i);
                                                break;
                                            }
                                        }

                                        match arg_index {
                                            Some(i) => {
                                                if let Some(mir_arg) = &mir_args[i] {
                                                    if let Some((_, span)) = given_keyword_arguments_[i] {
                                                        match repeated_arguments.entry(keyword) {
                                                            Entry::Occupied(mut e) => {
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: keyword_span,
                                                                    auxiliary: false,
                                                                    note: None,
                                                                });
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: span,
                                                                    auxiliary: false,
                                                                    note: None,
                                                                });
                                                            },
                                                            Entry::Vacant(e) => {
                                                                e.insert(vec![
                                                                    RenderableSpan {
                                                                        span: keyword_span,
                                                                        auxiliary: false,
                                                                        note: None,
                                                                    },
                                                                    RenderableSpan {
                                                                        span: span,
                                                                        auxiliary: false,
                                                                        note: None,
                                                                    },
                                                                ]);
                                                            },
                                                        }
                                                    }

                                                    else {
                                                        let keyword_str = keyword.unintern_or_default(&session.intermediate_dir);

                                                        match repeated_arguments.entry(keyword) {
                                                            Entry::Occupied(mut e) => {
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: keyword_span,
                                                                    auxiliary: false,
                                                                    note: None,
                                                                });
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: mir_arg.error_span_wide(),
                                                                    auxiliary: false,
                                                                    note: Some(format!("This argument is `{keyword_str}` because it's the {} argument.", to_ordinal(i + 1))),
                                                                });
                                                            },
                                                            Entry::Vacant(e) => {
                                                                e.insert(vec![
                                                                    RenderableSpan {
                                                                        span: keyword_span,
                                                                        auxiliary: false,
                                                                        note: None,
                                                                    },
                                                                    RenderableSpan {
                                                                        span: mir_arg.error_span_wide(),
                                                                        auxiliary: false,
                                                                        note: Some(format!("This argument is `{keyword_str}` because it's the {} argument.", to_ordinal(i + 1))),
                                                                    },
                                                                ]);
                                                            },
                                                        }
                                                    }
                                                }

                                                match Expr::from_hir(&hir_arg.arg, session) {
                                                    Ok(arg) => {
                                                        mir_args[i] = Some(arg);
                                                    },
                                                    Err(()) => {
                                                        has_error = true;
                                                    },
                                                }

                                                given_keyword_arguments_[i] = Some((keyword, keyword_span));
                                            },
                                            None => {
                                                session.errors.push(Error {
                                                    kind: ErrorKind::InvalidKeywordArgument(keyword),
                                                    spans: keyword_span.simple_error(),
                                                    note: None,
                                                });
                                                has_error = true;
                                            },
                                        }
                                    },
                                    None => {
                                        match Expr::from_hir(&hir_arg.arg, session) {
                                            Ok(arg) => {
                                                mir_args[positional_arg_cursor] = Some(arg);
                                            },
                                            Err(()) => {
                                                has_error = true;
                                            },
                                        }

                                        positional_arg_cursor += 1;
                                    },
                                }
                            }

                            for (keyword, error_spans) in repeated_arguments.into_iter() {
                                // remove repeats and sort by span
                                let mut error_spans = error_spans.into_iter().map(
                                    |span| (span.span, span)
                                ).collect::<HashMap<_, _>>().into_iter().map(
                                    |(_, span)| span
                                ).collect::<Vec<_>>();
                                error_spans.sort_by_key(|span| span.span);

                                session.errors.push(Error {
                                    kind: ErrorKind::KeywordArgumentRepeated(keyword),
                                    spans: error_spans,
                                    note: None,
                                });
                            }

                            for i in 0..params.len() {
                                match (&mir_args[i], &params[i].default_value) {
                                    (None, Some(default_value)) => {
                                        mir_args[i] = Some(Expr::Ident(*default_value));
                                    },
                                    _ => {},
                                }
                            }

                            let mut result = Vec::with_capacity(mir_args.len());
                            let mut g = Vec::with_capacity(mir_args.len());

                            for (i, mir_arg) in mir_args.into_iter().enumerate() {
                                if let Some(mir_arg) = mir_arg {
                                    result.push(mir_arg);

                                    if let Some((keyword, _)) = given_keyword_arguments_[i] {
                                        g.push((keyword, result.len() - 1));
                                    }
                                }

                                // If mir_arg is None, that's a compile error, but we're not raising an error yet.
                                // We'll raise an error after type-check/inference, so that we can add more information to the error message.
                            }

                            given_keyword_arguments = g;
                            Some(result)
                        },
                        None => None,
                    },
                    None => None,
                };

                // If we cannot access the exact definition of the func,
                // we can only process positional arguments and cannot do anything with the default values.
                if mir_args.is_none() {
                    mir_args = {
                        let mut result = Vec::with_capacity(hir_args.len());

                        for hir_arg in hir_args.iter() {
                            match hir_arg.keyword {
                                Some((_, keyword_span)) => {
                                    session.errors.push(Error {
                                        kind: ErrorKind::KeywordArgumentNotAllowed,
                                        spans: keyword_span.simple_error(),
                                        note: None,
                                    });
                                    has_error = true;
                                },
                                None => match Expr::from_hir(&hir_arg.arg, session) {
                                    Ok(arg) => {
                                        result.push(arg);
                                    },
                                    Err(()) => {
                                        has_error = true;
                                    },
                                },
                            }
                        }

                        Some(result)
                    };
                }

                let args = mir_args.unwrap();

                if has_error {
                    Err(())
                }

                else {
                    Ok(Expr::Call {
                        func,
                        args,
                        arg_group_span: *arg_group_span,
                        generic_defs,
                        given_keyword_arguments,
                    })
                }
            },
            // converts `f"{x} + {y} = {x + y}"` to `to_string(x) ++ " + " ++ to_string(y) ++ " = " ++ to_string(x + y)`
            hir::Expr::FormattedString { raw, elements: hir_elements, span: total_span } => {
                let mut has_error = false;
                let mut elements = Vec::with_capacity(hir_elements.len());

                // We don't do any optimizations here (e.g. skipping an empty string).
                // There's a dedicated optimization pass in the mir (WIP).
                for hir_element in hir_elements.iter() {
                    match hir_element {
                        hir::ExprOrString::String { s, span: curr_span } => {
                            let e = Expr::String {
                                binary: false,
                                s: *s,
                                // `total_span` includes quotes, but `curr_span` doesn't.
                                span: if hir_elements.len() == 1 { *total_span } else { *curr_span },
                            };

                            elements.push(e);
                        },
                        hir::ExprOrString::Expr(e) => match Expr::from_hir(e, session) {
                            Ok(e) => {
                                let derived_span = e.error_span_wide().derive(SpanDeriveKind::FStringToString);

                                // converts `x` to `to_string(x)`.
                                let e = Expr::Call {
                                    func: Callable::Static {
                                        def_span: session.get_lang_item_span("fn.to_string"),
                                        span: derived_span,
                                    },
                                    args: vec![e],
                                    arg_group_span: derived_span,
                                    generic_defs: vec![session.get_lang_item_span("fn.to_string.generic.0")],
                                    given_keyword_arguments: vec![],
                                };

                                elements.push(e);
                            },
                            Err(()) => {
                                has_error = true;
                            },
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    match elements.len() {
                        // is this possible?
                        0 => Ok(Expr::String {
                            binary: false,
                            s: InternedString::empty(),
                            span: *total_span,
                        }),
                        1 => Ok(elements.remove(0)),
                        _ => Ok(concat_strings(elements, session)),
                    }
                }
            },
            hir::Expr::List { elements, group_span } |
            hir::Expr::Tuple { elements, group_span } => {
                let mut has_error = false;
                let mut mir_elements = Vec::with_capacity(elements.len());
                let is_tuple = matches!(hir_expr, hir::Expr::Tuple { .. });

                for element in elements.iter() {
                    match Expr::from_hir(element, session) {
                        Ok(element) => {
                            mir_elements.push(element);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    let func = if is_tuple {
                        Callable::TupleInit { group_span: *group_span }
                    } else {
                        Callable::ListInit { group_span: *group_span }
                    };
                    Ok(Expr::Call {
                        func,
                        args: mir_elements,
                        arg_group_span: *group_span,

                        // TODO: It needs `generic_def` if it's `Callable::ListInit`
                        generic_defs: vec![],
                        given_keyword_arguments: vec![],
                    })
                }
            },
            hir::Expr::StructInit { r#struct, fields: hir_fields, group_span } => {
                let group_span = *group_span;
                let mut has_error = false;
                let (def_span, span) = match Expr::from_hir(r#struct, session)? {
                    expr @ Expr::Ident(id) => {
                        let (is_struct, explain) = match id.origin {
                            NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                                NameKind::Struct => (true, None),
                                NameKind::Let { .. } => (false, Some("a value")),
                                NameKind::Func => (false, Some("a function")),
                                NameKind::Enum => (false, Some("an enum")),

                                // It may or may not be a struct, but we don't know that...
                                NameKind::EnumVariant { .. } => todo!(),

                                NameKind::Alias => (false, Some("a type alias")),
                                NameKind::Module => (false, Some("a module")),
                                NameKind::Use => unreachable!(),
                                NameKind::FuncParam => (false, Some("a function parameter")),
                                NameKind::Generic => (false, Some("a generic parameter")),
                                NameKind::PatternNameBind => (false, Some("a pattern name bind")),
                                NameKind::Pipeline => (false, Some("a piped value")),
                            },
                            NameOrigin::FuncParam { .. } => (false, Some("a function parameter")),
                            NameOrigin::Generic { .. } => (false, Some("a generic parameter")),
                            NameOrigin::External => unreachable!(),
                        };

                        if !is_struct {
                            session.errors.push(Error {
                                kind: ErrorKind::NotStruct { id: Some(id) },
                                spans: expr.error_span_wide().simple_error(),
                                note: Some(format!("This is {}, not a struct.", explain.unwrap())),
                            });
                            return Err(());
                        }

                        (id.def_span, id.span)
                    },
                    expr => {
                        session.errors.push(Error {
                            kind: ErrorKind::NotStruct { id: None },
                            spans: expr.error_span_wide().simple_error(),
                            note: None,
                        });
                        return Err(());
                    },
                };
                let mut generic_defs = vec![];
                // for better error messages
                let mut repeated_fields: HashMap<InternedString, Vec<RenderableSpan>> = HashMap::new();

                match session.struct_shapes.get(&def_span) {
                    Some(struct_shape) => {
                        if !struct_shape.generics.is_empty() {
                            for generic_def in struct_shape.generics.iter() {
                                session.generic_instances.insert(
                                    (r#struct.error_span_wide(), generic_def.name_span),
                                    Type::GenericInstance {
                                        call: r#struct.error_span_wide(),
                                        generic: generic_def.name_span,
                                    },
                                );
                                generic_defs.push(generic_def.name_span);
                            }
                        }

                        let field_defs = struct_shape.fields.clone();
                        let mut mir_fields = vec![None; hir_fields.len()];
                        let mut name_spans = vec![None; hir_fields.len()];

                        for hir_field in hir_fields.iter() {
                            let mut field_index = None;

                            for (i, field_def) in field_defs.iter().enumerate() {
                                if field_def.name == hir_field.name {
                                    field_index = Some(i);
                                    break;
                                }
                            }

                            match field_index {
                                Some(i) => {
                                    if mir_fields[i].is_some() {
                                        match repeated_fields.entry(hir_field.name) {
                                            Entry::Occupied(mut e) => {
                                                e.get_mut().push(RenderableSpan {
                                                    span: hir_field.name_span,
                                                    auxiliary: false,
                                                    note: None,
                                                });
                                                e.get_mut().push(RenderableSpan {
                                                    span: name_spans[i].unwrap(),
                                                    auxiliary: false,
                                                    note: None,
                                                });
                                            },
                                            Entry::Vacant(e) => {
                                                e.insert(vec![
                                                    RenderableSpan {
                                                        span: hir_field.name_span,
                                                        auxiliary: false,
                                                        note: None,
                                                    },
                                                    RenderableSpan {
                                                        span: name_spans[i].unwrap(),
                                                        auxiliary: false,
                                                        note: None,
                                                    },
                                                ]);
                                            },
                                        }
                                    }

                                    match Expr::from_hir(&hir_field.value, session) {
                                        Ok(field) => {
                                            mir_fields[i] = Some(field);
                                        },
                                        Err(()) => {
                                            has_error = true;
                                        },
                                    }

                                    name_spans[i] = Some(hir_field.name_span);
                                },
                                None => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::InvalidStructField(hir_field.name),
                                        spans: hir_field.name_span.simple_error(),
                                        note: None,
                                    });
                                },
                            }
                        }

                        for (field_name, error_spans) in repeated_fields.into_iter() {
                            // remove repeats and sort by span
                            let mut error_spans = error_spans.into_iter().map(
                                |span| (span.span, span)
                            ).collect::<HashMap<_, _>>().into_iter().map(
                                |(_, span)| span
                            ).collect::<Vec<_>>();
                            error_spans.sort_by_key(|span| span.span);

                            session.errors.push(Error {
                                kind: ErrorKind::StructFieldRepeated(field_name),
                                spans: error_spans,
                                note: None,
                            });
                        }

                        for i in 0..field_defs.len() {
                            match (&mir_fields[i], &field_defs[i].default_value) {
                                (None, Some(default_value)) => {
                                    mir_fields[i] = Some(Expr::Ident(*default_value));
                                },
                                _ => {},
                            }
                        }

                        let mut mir_fields_unwrapped = Vec::with_capacity(mir_fields.len());

                        for (i, mir_field) in mir_fields.into_iter().enumerate() {
                            match mir_field {
                                Some(mir_field) => {
                                    mir_fields_unwrapped.push(mir_field);
                                },
                                None => {
                                    let field_name = field_defs[i].name.unintern_or_default(&session.intermediate_dir);
                                    session.errors.push(Error {
                                        kind: ErrorKind::MissingStructField(field_defs[i].name),
                                        spans: vec![
                                            RenderableSpan {
                                                span: group_span,
                                                auxiliary: false,
                                                note: Some(format!("This instance is missing field `{field_name}`.")),
                                            },
                                            RenderableSpan {
                                                span: field_defs[i].name_span,
                                                auxiliary: true,
                                                note: Some(format!("The field `{field_name}` is defined here.")),
                                            },
                                        ],
                                        note: None,
                                    });
                                    has_error = true;
                                },
                            }
                        }

                        if has_error {
                            Err(())
                        }

                        else {
                            Ok(Expr::Call {
                                func: Callable::StructInit {
                                    def_span,
                                    span,
                                },
                                args: mir_fields_unwrapped,
                                arg_group_span: group_span,
                                generic_defs,
                                given_keyword_arguments: vec![],
                            })
                        }
                    },
                    // It already checked the def_span. If `struct_shapes` doesn't have this span, that's an ICE.
                    None => unreachable!(),
                }
            },
            hir::Expr::Path { lhs, fields } => match Expr::from_hir(lhs, session) {
                Ok(lhs) => Ok(Expr::Path {
                    lhs: Box::new(lhs),
                    fields: fields.clone(),
                }),
                Err(()) => Err(()),
            },
            hir::Expr::FieldModifier { fields, lhs, rhs } => match (
                Expr::from_hir(lhs, session),
                Expr::from_hir(rhs, session),
            ) {
                (Ok(lhs), Ok(rhs)) => Ok(Expr::FieldModifier {
                    fields: fields.clone(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                _ => Err(()),
            },
            hir::Expr::PrefixOp { op, op_span, rhs } => {
                let func = Callable::Static {
                    def_span: session.get_lang_item_span(op.get_def_lang_item()),
                    span: *op_span,
                };
                let generic_defs = op.get_generic_lang_items().iter().map(
                    |lang_item| session.get_lang_item_span(lang_item)
                ).collect();

                Ok(Expr::Call {
                    func,
                    args: vec![Expr::from_hir(rhs, session)?],
                    arg_group_span: rhs.error_span_wide().derive(SpanDeriveKind::Trivial),
                    generic_defs,
                    given_keyword_arguments: vec![],
                })
            },
            hir::Expr::InfixOp { op, op_span, lhs, rhs } => {
                // `hir::Expr`'s span has more information that `mir::Expr`'s span.
                let expr_span = lhs.error_span_wide().merge(*op_span).merge(rhs.error_span_wide()).derive(SpanDeriveKind::Trivial);

                match (
                    Expr::from_hir(lhs, session),
                    Expr::from_hir(rhs, session),
                ) {
                    (Ok(lhs), Ok(rhs)) => {
                        match op {
                            // `lhs && rhs` -> `if lhs { rhs } else { False }`
                            InfixOp::LogicAnd => Ok(Expr::If(If {
                                if_span: *op_span,
                                cond: Box::new(lhs),
                                else_span: Span::None,
                                true_value: Box::new(rhs),
                                true_group_span: expr_span,
                                false_value: Box::new(Expr::Ident(IdentWithOrigin {
                                    id: intern_string(b"False", &session.intermediate_dir).unwrap(),
                                    span: Span::None,
                                    origin: NameOrigin::Foreign {
                                        kind: NameKind::EnumVariant {
                                            parent: session.get_lang_item_span("type.Bool"),
                                        },
                                    },
                                    def_span: session.get_lang_item_span("variant.Bool.False"),
                                })),
                                false_group_span: expr_span,
                                from_short_circuit: Some(ShortCircuitKind::And),
                            })),
                            // `lhs || rhs` -> `if lhs { True } else { rhs }`
                            InfixOp::LogicOr => Ok(Expr::If(If {
                                if_span: *op_span,
                                cond: Box::new(lhs),
                                else_span: Span::None,
                                true_value: Box::new(Expr::Ident(IdentWithOrigin {
                                    id: intern_string(b"True", &session.intermediate_dir).unwrap(),
                                    span: Span::None,
                                    origin: NameOrigin::Foreign {
                                        kind: NameKind::EnumVariant {
                                            parent: session.get_lang_item_span("type.Bool"),
                                        },
                                    },
                                    def_span: session.get_lang_item_span("variant.Bool.True"),
                                })),
                                true_group_span: expr_span,
                                false_value: Box::new(rhs),
                                false_group_span: expr_span,
                                from_short_circuit: Some(ShortCircuitKind::Or),
                            })),
                            _ => {
                                let func = Callable::Static {
                                    def_span: session.get_lang_item_span(op.get_def_lang_item()),
                                    span: *op_span,
                                };
                                let generic_defs = op.get_generic_lang_items().iter().map(
                                    |lang_item| session.get_lang_item_span(lang_item)
                                ).collect();

                                Ok(Expr::Call {
                                    func,
                                    args: vec![lhs, rhs],
                                    arg_group_span: expr_span,
                                    generic_defs,
                                    given_keyword_arguments: vec![],
                                })
                            },
                        }
                    },
                    _ => Err(()),
                }
            },
            hir::Expr::PostfixOp { op, op_span, lhs } => {
                let func = Callable::Static {
                    def_span: session.get_lang_item_span(op.get_def_lang_item()),
                    span: *op_span,
                };
                let generic_defs = op.get_generic_lang_items().iter().map(
                    |lang_item| session.get_lang_item_span(lang_item)
                ).collect();

                Ok(Expr::Call {
                    func,
                    args: vec![Expr::from_hir(lhs, session)?],
                    arg_group_span: lhs.error_span_wide().derive(SpanDeriveKind::Trivial),
                    generic_defs,
                    given_keyword_arguments: vec![],
                })
            },
        }
    }

    pub fn dummy() -> Expr {
        Expr::Char { ch: 0, span: Span::None }
    }

    pub fn error_span_narrow(&self) -> Span {
        match self {
            Expr::Ident(id) => id.span,
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } => *span,
            Expr::If(r#if) => r#if.if_span,
            Expr::Match(r#match) => r#match.keyword_span,
            Expr::Block(block) => block.group_span,
            // Let's hope it doesn't panic...
            Expr::Path { fields, .. } => fields[0].dot_span().unwrap(),
            Expr::FieldModifier { fields, .. } => {
                let mut merged_span = fields[0].1;

                for (_, span) in fields.iter() {
                    merged_span = merged_span.merge(*span);
                }

                merged_span
            },
            Expr::Call { func, .. } => func.error_span_narrow(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Expr::Ident(id) => id.span,
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } => *span,
            Expr::If(r#if) => r#if.if_span.merge(r#if.true_group_span).merge(r#if.false_group_span),
            Expr::Match(r#match) => r#match.keyword_span.merge(r#match.scrutinee.error_span_wide()).merge(r#match.group_span),
            Expr::Block(block) => block.group_span,
            Expr::Path { lhs, fields } => {
                let mut span = lhs.error_span_wide();

                for field in fields.iter() {
                    match field {
                        Field::Name { span: s, .. } => {
                            span = span.merge(*s);
                        },
                        _ => unreachable!(),
                    }
                }

                span
            },
            Expr::FieldModifier { lhs, fields, rhs } => {
                let mut span = lhs.error_span_wide();

                for (_, field_span) in fields.iter() {
                    span = span.merge(*field_span);
                }

                span.merge(rhs.error_span_wide())
            },
            Expr::Call { func, arg_group_span, .. } => func.error_span_wide().merge(*arg_group_span),
        }
    }
}

impl Callable {
    pub fn error_span_narrow(&self) -> Span {
        match self {
            Callable::Static { span, .. } |
            Callable::StructInit { span, .. } |
            Callable::TupleInit { group_span: span } |
            Callable::ListInit { group_span: span } => *span,
            Callable::Dynamic(expr) => expr.error_span_narrow(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Callable::Static { span, .. } |
            Callable::StructInit { span, .. } |
            Callable::TupleInit { group_span: span } |
            Callable::ListInit { group_span: span } => *span,
            Callable::Dynamic(expr) => expr.error_span_wide(),
        }
    }
}

// TODO: can we do some optimizations here?
fn concat_strings(mut strings: Vec<Expr>, session: &Session) -> Expr {
    let def_span = session.get_lang_item_span(InfixOp::Concat.get_def_lang_item());
    let generic_defs = InfixOp::Concat.get_generic_lang_items().iter().map(
        |lang_item| session.get_lang_item_span(lang_item)
    ).collect();

    match strings.len() {
        0 | 1 => unreachable!(),
        2 => {
            let rhs = strings.pop().unwrap();
            let lhs = strings.pop().unwrap();
            let derived_span = rhs.error_span_wide().derive(SpanDeriveKind::FStringConcat);

            Expr::Call {
                func: Callable::Static {
                    def_span,
                    span: derived_span,
                },
                args: vec![lhs, rhs],
                arg_group_span: derived_span,
                generic_defs,
                given_keyword_arguments: vec![],
            }
        },
        _ => {
            let tail = strings.pop().unwrap();
            let head = concat_strings(strings, session);
            let derived_span = tail.error_span_wide().derive(SpanDeriveKind::FStringConcat);

            Expr::Call {
                func: Callable::Static {
                    def_span,
                    span: derived_span,
                },
                args: vec![head, tail],
                arg_group_span: derived_span,
                generic_defs,
                given_keyword_arguments: vec![],
            }
        },
    }
}
