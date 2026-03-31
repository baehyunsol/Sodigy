use crate::{
    Block,
    Dotfish,
    If,
    Match,
    Session,
    Type,
    lower_hir_if,
};
use sodigy_error::{EnumFieldKind, Error, ErrorKind, NotXBut, comma_list_strs, to_ordinal};
use sodigy_hir::{self as hir, EnumVariantFields, Generic};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{Field, merge_field_spans};
use sodigy_session::SodigySession;
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{Constant, InfixOp};
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

mod dispatch;

#[derive(Clone, Debug)]
pub enum Expr {
    Ident {
        id: IdentWithOrigin,
        dotfish: Option<Dotfish>,
    },
    Constant(Constant),
    If(If),

    // `Match` is later lowered to a `Block`.
    Match(Match),
    Block(Block),
    Field {
        lhs: Box<Expr>,
        fields: Vec<Field>,
        dotfish: Vec<Option<Dotfish>>,
    },
    FieldUpdate {
        fields: Vec<Field>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        func: Callable,
        args: Vec<Expr>,
        arg_group_span: Span,

        // It's lowered from dotfish operators.
        // Only `Callable::Static`, `Callable::StructInit` and `Callable::EnumInit` can have this.
        types: Option<Dotfish>,

        // It helps generating error messages.
        // It has type `Vec<(keyword: InternedString, n: usize)>` where
        // nth argument in `args` has keyword `keyword`.
        given_keyword_args: Vec<(InternedString, usize)>,
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
    EnumInit {
        parent_def_span: Span,
        variant_def_span: Span,
        kind: EnumFieldKind,
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
            hir::Expr::Path(path) => {
                // inter-hir's `check_expr_path` should guarantee this
                assert!(path.fields.is_empty());
                assert!(path.dotfish.len() == 1);
                Ok(Expr::from_ident_with_origin(
                    &path.id,
                    Dotfish::from_hir(path.dotfish.last().unwrap(), session)?,
                ))
            },
            hir::Expr::Constant(c) => Ok(Expr::Constant(c.clone())),

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
                let mut given_keyword_args = vec![];
                let mut dotfish = None;

                // TODO: This is the right place to lower dotfish operators.

                let (call_span, func) = match Expr::from_hir(func, session) {
                    Ok(ref e @ Expr::Ident { ref id, dotfish: ref hir_dotfish }) => {
                        if hir_dotfish.is_some() {
                            dotfish = hir_dotfish.clone();
                        }

                        match &id.origin {
                            NameOrigin::Local { kind } |
                            NameOrigin::Foreign { kind } => match kind {
                                NameKind::Func => {
                                    def_span = Some(id.def_span.clone());
                                    (
                                        id.span.clone(),
                                        Callable::Static {
                                            def_span: id.def_span.clone(),
                                            span: id.span.clone(),
                                        },
                                    )
                                },
                                // The programmer defines a functor using `let` keyword
                                // and calls it. In this case, we have to dynamically call the
                                // function on runtime. (Maybe we can do some optimizations and turn it into a static call?)
                                NameKind::Let { .. } => {
                                    def_span = Some(id.def_span.clone());
                                    (id.span.clone(), Callable::Dynamic(Box::new(e.clone())))
                                },
                                _ => panic!("TODO: {kind:?}"),
                            },
                            NameOrigin::FuncParam { .. } => (id.span.clone(), Callable::Dynamic(Box::new(e.clone()))),
                            NameOrigin::GenericParam { .. } => unreachable!(),
                            NameOrigin::External => unreachable!(),
                        }
                    },
                    // call_span has to be the name_span of the last field, because `get_type_of_field` works this way
                    Ok(Expr::Field { lhs, fields, dotfish: hir_dotfish }) => {
                        dotfish = hir_dotfish.last().unwrap().clone();

                        (
                            fields.last().unwrap().unwrap_name_span(),
                            Callable::Dynamic(Box::new(Expr::Field { lhs, dotfish: vec![None; fields.len() + 1], fields })),
                        )
                    },
                    Ok(Expr::Call { func: Callable::EnumInit { parent_def_span, variant_def_span, kind, span }, arg_group_span, types, .. }) => match kind {
                        EnumFieldKind::None => (
                            span.clone(),
                            Callable::EnumInit {
                                parent_def_span,
                                variant_def_span,
                                kind: EnumFieldKind::Tuple,
                                span,
                            },
                        ),
                        _ => {
                            session.errors.push(todo!());
                            return Err(());
                        },
                    },
                    Ok(func) => (func.error_span_wide(), Callable::Dynamic(Box::new(func))),
                    Err(()) => {
                        has_error = true;

                        // It's already an error, but we want to find as many errors as possible.
                        for hir::CallArg { arg, .. } in hir_args.iter() {
                            let _ = Expr::from_hir(arg, session);
                        }

                        return Err(());
                    },
                };

                let mut generics: Option<Vec<Generic>> = None;

                // It processes the keyword arguments and default values, if it finds ones.
                let mut mir_args: Option<Vec<Expr>> = match def_span {
                    Some(def_span) => {
                        if let Some(func_shape) = session.global_context.func_shapes.unwrap().get(&def_span) {
                            if !func_shape.generics.is_empty() {
                                generics = Some(func_shape.generics.clone());
                            }

                            let params = func_shape.params.to_vec();
                            let mut mir_args: Vec<Option<Expr>> = vec![None; params.len().max(hir_args.len())];

                            // used for error messages
                            let mut given_keyword_args_: Vec<Option<(InternedString, Span)>> = vec![None; params.len().max(hir_args.len())];

                            // Positional args cannot come after a keyword arg, and hir guarantees that.
                            let mut positional_arg_cursor = 0;

                            // Another attempt for even better error messages
                            let mut repeated_args: HashMap<InternedString, Vec<RenderableSpan>> = HashMap::new();

                            for hir_arg in hir_args.iter() {
                                match &hir_arg.keyword {
                                    Some((keyword, keyword_span)) => {
                                        let mut arg_index = None;

                                        // It's O(n), but n is very small
                                        for (i, param) in params.iter().enumerate() {
                                            if param.name == *keyword {
                                                arg_index = Some(i);
                                                break;
                                            }
                                        }

                                        match arg_index {
                                            Some(i) => {
                                                if let Some(mir_arg) = &mir_args[i] {
                                                    if let Some((_, span)) = &given_keyword_args_[i] {
                                                        match repeated_args.entry(*keyword) {
                                                            Entry::Occupied(mut e) => {
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: keyword_span.clone(),
                                                                    auxiliary: false,
                                                                    note: None,
                                                                });
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: span.clone(),
                                                                    auxiliary: false,
                                                                    note: None,
                                                                });
                                                            },
                                                            Entry::Vacant(e) => {
                                                                e.insert(vec![
                                                                    RenderableSpan {
                                                                        span: keyword_span.clone(),
                                                                        auxiliary: false,
                                                                        note: None,
                                                                    },
                                                                    RenderableSpan {
                                                                        span: span.clone(),
                                                                        auxiliary: false,
                                                                        note: None,
                                                                    },
                                                                ]);
                                                            },
                                                        }
                                                    }

                                                    else {
                                                        let keyword_str = keyword.unintern_or_default(&session.intermediate_dir);

                                                        match repeated_args.entry(*keyword) {
                                                            Entry::Occupied(mut e) => {
                                                                e.get_mut().push(RenderableSpan {
                                                                    span: keyword_span.clone(),
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
                                                                        span: keyword_span.clone(),
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

                                                given_keyword_args_[i] = Some((*keyword, keyword_span.clone()));
                                            },
                                            None => {
                                                session.errors.push(Error {
                                                    kind: ErrorKind::InvalidKeywordArg(*keyword),
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

                            for (keyword, error_spans) in repeated_args.into_iter() {
                                // remove repeats and sort by span
                                let mut error_spans = error_spans.into_iter().map(
                                    |span| (span.span.clone(), span)
                                ).collect::<HashMap<_, _>>().into_iter().map(
                                    |(_, span)| span
                                ).collect::<Vec<_>>();
                                error_spans.sort_by_key(|span| span.span.clone());

                                session.errors.push(Error {
                                    kind: ErrorKind::KeywordArgRepeated(keyword),
                                    spans: error_spans,
                                    note: None,
                                });
                            }

                            for i in 0..params.len() {
                                match (&mir_args[i], &params[i].default_value) {
                                    (None, Some(default_value)) => {
                                        mir_args[i] = Some(Expr::from_ident_with_origin(default_value, None));
                                    },
                                    _ => {},
                                }
                            }

                            let mut result = Vec::with_capacity(mir_args.len());
                            let mut g = Vec::with_capacity(mir_args.len());

                            for (i, mir_arg) in mir_args.into_iter().enumerate() {
                                if let Some(mir_arg) = mir_arg {
                                    result.push(mir_arg);

                                    if let Some((keyword, _)) = &given_keyword_args_[i] {
                                        g.push((*keyword, result.len() - 1));
                                    }
                                }

                                // If mir_arg is None, that's a compile error, but we're not raising an error yet.
                                // We'll raise an error after type-check/inference, so that we can add more information to the error message.
                            }

                            given_keyword_args = g;
                            Some(result)
                        }

                        else if let Some(enum_shape) = session.global_context.enum_shapes.unwrap().get(&def_span) {
                            if !enum_shape.generics.is_empty() {
                                generics = Some(enum_shape.generics.clone());
                            }

                            None
                        }

                        else {
                            None
                        }
                    },
                    None => None,
                };

                if let Some(generics) = &generics {
                    for generic in generics.iter() {
                        session.generic_args.insert(
                            (call_span.clone(), generic.name_span.clone()),
                            Type::GenericArg {
                                call: call_span.clone(),
                                generic: generic.name_span.clone(),
                            },
                        );
                    }
                }

                // If we cannot access the exact definition of the func,
                // we can only process positional arguments and cannot do anything with the default values.
                if mir_args.is_none() {
                    mir_args = {
                        let mut result = Vec::with_capacity(hir_args.len());

                        for hir_arg in hir_args.iter() {
                            match &hir_arg.keyword {
                                Some((_, keyword_span)) => {
                                    session.errors.push(Error {
                                        kind: ErrorKind::KeywordArgNotAllowed,
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
                        arg_group_span: arg_group_span.clone(),
                        types: dotfish,
                        given_keyword_args,
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
                            let e = Expr::Constant(Constant::String {
                                binary: false,
                                s: *s,
                                // `total_span` includes quotes, but `curr_span` doesn't.
                                span: if hir_elements.len() == 1 { total_span.clone() } else { curr_span.clone() },
                            });

                            elements.push(e);
                        },
                        hir::ExprOrString::Expr(e) => match Expr::from_hir(e, session) {
                            Ok(e) => {
                                let derived_span = e.error_span_wide().derive(SpanDeriveKind::FStringToString);

                                // converts `x` to `convert.<_, String>(x)`.
                                let e = Expr::Call {
                                    func: Callable::Static {
                                        def_span: session.get_lang_item_span("fn.convert"),
                                        span: derived_span.clone(),
                                    },
                                    args: vec![e],
                                    arg_group_span: derived_span.clone(),
                                    types: Some(Dotfish {
                                        types: vec![
                                            Type::Var { def_span: derived_span.clone(), is_return: false },
                                            Type::Data {
                                                constructor_def_span: session.get_lang_item_span("type.List"),
                                                constructor_span: derived_span.clone(),
                                                args: Some(vec![Type::Data {
                                                    constructor_def_span: session.get_lang_item_span("type.Char"),
                                                    constructor_span: derived_span.clone(),
                                                    args: None,
                                                    group_span: None,
                                                }]),
                                                group_span: Some(Span::None),
                                            },
                                        ],
                                        group_span: Span::None,
                                    }),
                                    given_keyword_args: vec![],
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
                        0 => Ok(Expr::Constant(Constant::String {
                            binary: false,
                            s: InternedString::empty(),
                            span: total_span.clone(),
                        })),
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
                        Callable::TupleInit { group_span: group_span.clone() }
                    } else {
                        Callable::ListInit { group_span: group_span.clone() }
                    };
                    Ok(Expr::Call {
                        func,
                        args: mir_elements,
                        arg_group_span: group_span.clone(),
                        types: None,
                        given_keyword_args: vec![],
                    })
                }
            },
            // Unlike hir::Expr::Call, we'll raise an error here if the number of
            // fields is wrong.
            hir::Expr::StructInit { constructor, fields: hir_fields, group_span } => {
                let group_span = group_span.clone();
                let mut has_error = false;
                let enum_parent_def_span = match &constructor.id.origin {
                    NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                        NameKind::EnumVariant { parent } => Some(parent.clone()),
                        _ => None,
                    },
                    _ => None,
                };

                let (def_span, call_span) = (constructor.id.def_span.clone(), constructor.id.span.clone());

                // TODO: it has to lower dotfish operators

                let (field_defs, generics, struct_name, is_enum_variant) = if let Some(struct_shape) = session.global_context.struct_shapes.unwrap().get(&def_span) {
                    (&struct_shape.fields, &struct_shape.generics, struct_shape.name, false)
                }

                else if let Some(enum_parent_def_span) = &enum_parent_def_span && let Some(enum_shape) = session.global_context.enum_shapes.unwrap().get(enum_parent_def_span) {
                    let variant = &enum_shape.variants[*enum_shape.variant_index.get(&def_span).unwrap()];
                    let fields = match &variant.fields {
                        EnumVariantFields::Struct(fields) => fields,
                        f => {
                            session.errors.push(Error {
                                kind: ErrorKind::MismatchedEnumFieldKind {
                                    expected: f.into(),
                                    got: EnumFieldKind::Struct,
                                },
                                spans: call_span.simple_error(),
                                note: None,
                            });
                            return Err(());
                        },
                    };
                    let name = {
                        let enum_name = enum_shape.name.unintern_or_default(&session.intermediate_dir);
                        let variant_name = variant.name.unintern_or_default(&session.intermediate_dir);
                        intern_string(format!("{enum_name}.{variant_name}").as_bytes(), &session.intermediate_dir).unwrap()
                    };

                    (fields, &enum_shape.generics, name, true)
                }

                else {
                    let but = match &constructor.id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => kind.into(),
                        _ => NotXBut::Expr,
                    };
                    session.errors.push(Error {
                        kind: ErrorKind::NotStruct { id: constructor.id.id, tuple_struct: false, but },
                        spans: call_span.simple_error(),
                        note: None,
                    });

                    return Err(());
                };

                // for better error messages
                let mut repeated_fields: HashMap<InternedString, Vec<RenderableSpan>> = HashMap::new();

                let mut missing_fields = vec![];
                let mut invalid_fields = vec![];

                if !generics.is_empty() {
                    for generic in generics.iter() {
                        session.generic_args.insert(
                            (call_span.clone(), generic.name_span.clone()),
                            Type::GenericArg {
                                call: call_span.clone(),
                                generic: generic.name_span.clone(),
                            },
                        );
                    }
                }

                let mut mir_fields = vec![None; field_defs.len()];
                let mut name_spans = vec![None; field_defs.len()];
                let mut mir_fields_final = Vec::with_capacity(mir_fields.len());

                if hir_fields.len() > field_defs.len() {
                    let field_names = field_defs.iter().map(|field| field.name).collect::<HashSet<_>>();

                    for hir_field in hir_fields.iter() {
                        match repeated_fields.entry(hir_field.name) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().push(RenderableSpan {
                                    span: hir_field.name_span.clone(),
                                    auxiliary: false,
                                    note: None,
                                });
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![
                                    RenderableSpan {
                                        span: hir_field.name_span.clone(),
                                        auxiliary: false,
                                        note: None,
                                    },
                                ]);
                            },
                        }

                        if !field_names.contains(&hir_field.name) {
                            invalid_fields.push((hir_field.name, hir_field.name_span.clone()));
                        }
                    }

                    has_error = true;
                }

                else {
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
                                                span: hir_field.name_span.clone(),
                                                auxiliary: false,
                                                note: None,
                                            });
                                            e.get_mut().push(RenderableSpan {
                                                span: name_spans[i].clone().unwrap(),
                                                auxiliary: false,
                                                note: None,
                                            });
                                        },
                                        Entry::Vacant(e) => {
                                            e.insert(vec![
                                                RenderableSpan {
                                                    span: hir_field.name_span.clone(),
                                                    auxiliary: false,
                                                    note: None,
                                                },
                                                RenderableSpan {
                                                    span: name_spans[i].clone().unwrap(),
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

                                name_spans[i] = Some(hir_field.name_span.clone());
                            },
                            None => {
                                invalid_fields.push((hir_field.name, hir_field.name_span.clone()));
                                has_error = true;
                            },
                        }
                    }

                    for (field_name, error_spans) in repeated_fields.into_iter() {
                        // remove repeats and sort by span
                        let mut error_spans = error_spans.into_iter().map(
                            |span| (span.span.clone(), span)
                        ).collect::<HashMap<_, _>>().into_iter().map(
                            |(_, span)| span
                        ).collect::<Vec<_>>();
                        error_spans.sort_by_key(|span| span.span.clone());

                        session.errors.push(Error {
                            kind: ErrorKind::StructFieldRepeated(field_name),
                            spans: error_spans,
                            note: None,
                        });
                        has_error = true;
                    }

                    for i in 0..field_defs.len() {
                        match (&mir_fields[i], &field_defs[i].default_value) {
                            (None, Some(default_value)) => {
                                mir_fields[i] = Some(Expr::from_ident_with_origin(default_value, None));
                            },
                            _ => {},
                        }
                    }

                    for (i, mir_field) in mir_fields.into_iter().enumerate() {
                        match mir_field {
                            Some(mir_field) => {
                                mir_fields_final.push(mir_field);
                            },
                            None => {
                                missing_fields.push(&field_defs[i]);
                                has_error = true;
                            },
                        }
                    }
                }

                if has_error {
                    if !missing_fields.is_empty() {
                        let names = missing_fields.iter().map(|field| field.name).collect::<Vec<_>>();
                        let mut spans = missing_fields.iter().map(
                            |field| RenderableSpan {
                                span: field.name_span.clone(),
                                auxiliary: true,
                                note: Some(format!(
                                    "Field `{}` is defined here.",
                                    field.name.unintern_or_default(&session.intermediate_dir),
                                )),
                            }
                        ).collect::<Vec<_>>();
                        spans.push(RenderableSpan {
                            span: group_span.clone(),
                            auxiliary: false,
                            note: Some(format!(
                                "Field{} {} {} missing here.",
                                if missing_fields.len() == 1 { "" } else { "s" },
                                comma_list_strs(
                                    &names.iter().map(|name| name.unintern_or_default(&session.intermediate_dir)).collect::<Vec<_>>(),
                                    "`",
                                    "`",
                                    "and",
                                ),
                                if missing_fields.len() == 1 { "is" } else { "are" },
                            )),
                        });

                        session.errors.push(Error {
                            kind: ErrorKind::MissingStructFields { struct_name, is_enum_variant, missing_fields: names },
                            spans,
                            note: None,
                        });
                    }

                    if !invalid_fields.is_empty() {
                        let names = invalid_fields.iter().map(|(name, _)| *name).collect();
                        let spans = invalid_fields.iter().map(
                            |(_, name_span)| RenderableSpan {
                                span: name_span.clone(),
                                auxiliary: false,
                                note: None,
                            }
                        ).collect::<Vec<_>>();

                        session.errors.push(Error {
                            kind: ErrorKind::InvalidStructFields { struct_name, is_enum_variant, invalid_fields: names },
                            spans,
                            note: Some(format!(
                                "Available field{} {} {}.",
                                if field_defs.len() == 1 { "" } else { "s" },
                                if field_defs.len() == 1 { "is" } else { "are" },
                                comma_list_strs(
                                    &field_defs.iter().map(|field| field.name.unintern_or_default(&session.intermediate_dir)).collect::<Vec<_>>(),
                                    "`",
                                    "`",
                                    "and",
                                ),
                            )),
                        });
                    }

                    Err(())
                }

                else {
                    Ok(Expr::Call {
                        func: Callable::StructInit {
                            def_span,
                            span: call_span,
                        },
                        args: mir_fields_final,
                        arg_group_span: group_span,
                        types: None,
                        given_keyword_args: vec![],
                    })
                }
            },
            hir::Expr::Field { lhs, fields, dotfish: hir_dotfish } => match Expr::from_hir(lhs, session) {
                Ok(lhs) => {
                    let mut dotfish = Vec::with_capacity(hir_dotfish.len());
                    let mut has_error = false;

                    for hir_dotfish in hir_dotfish.iter() {
                        match Dotfish::from_hir(hir_dotfish, session) {
                            Ok(d) => {
                                dotfish.push(d);
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
                        Ok(Expr::Field {
                            lhs: Box::new(lhs),
                            fields: fields.clone(),
                            dotfish,
                        })
                    }
                },
                Err(()) => Err(()),
            },
            hir::Expr::FieldUpdate { fields, lhs, rhs } => match (
                Expr::from_hir(lhs, session),
                Expr::from_hir(rhs, session),
            ) {
                (Ok(lhs), Ok(rhs)) => Ok(Expr::FieldUpdate {
                    fields: fields.clone(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                _ => Err(()),
            },
            hir::Expr::PrefixOp { op, op_span, rhs } => {
                let func = Callable::Static {
                    def_span: session.get_lang_item_span(op.get_def_lang_item()),
                    span: op_span.clone(),
                };

                Ok(Expr::Call {
                    func,
                    args: vec![Expr::from_hir(rhs, session)?],
                    arg_group_span: rhs.error_span_wide(),
                    types: None,
                    given_keyword_args: vec![],
                })
            },
            hir::Expr::InfixOp { op, op_span, lhs, rhs } => {
                // `hir::Expr`'s span has more information than `mir::Expr`'s span.
                let expr_span = lhs.error_span_wide().merge(op_span).merge(&rhs.error_span_wide());

                match (
                    Expr::from_hir(lhs, session),
                    Expr::from_hir(rhs, session),
                ) {
                    (Ok(lhs), Ok(rhs)) => {
                        match op {
                            // `lhs && rhs` -> `if lhs { rhs } else { False }`
                            InfixOp::LogicAnd => Ok(Expr::If(If {
                                if_span: op_span.clone(),
                                cond: Box::new(lhs),
                                else_span: Span::None,
                                true_value: Box::new(rhs),
                                true_group_span: expr_span.clone(),
                                false_value: Box::new(false_value(session)),
                                false_group_span: expr_span.clone(),
                                from_short_circuit: Some(ShortCircuitKind::And),
                            })),
                            // `lhs || rhs` -> `if lhs { True } else { rhs }`
                            InfixOp::LogicOr => Ok(Expr::If(If {
                                if_span: op_span.clone(),
                                cond: Box::new(lhs),
                                else_span: Span::None,
                                true_value: Box::new(true_value(session)),
                                true_group_span: expr_span.clone(),
                                false_value: Box::new(rhs),
                                false_group_span: expr_span.clone(),
                                from_short_circuit: Some(ShortCircuitKind::Or),
                            })),
                            _ => {
                                let func = Callable::Static {
                                    def_span: session.get_lang_item_span(op.get_def_lang_item()),
                                    span: op_span.clone(),
                                };

                                Ok(Expr::Call {
                                    func,
                                    args: vec![lhs, rhs],
                                    arg_group_span: expr_span,
                                    types: None,
                                    given_keyword_args: vec![],
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
                    span: op_span.clone(),
                };

                Ok(Expr::Call {
                    func,
                    args: vec![Expr::from_hir(lhs, session)?],
                    arg_group_span: lhs.error_span_wide(),
                    types: None,
                    given_keyword_args: vec![],
                })
            },
            hir::Expr::TypeConversion { keyword_span, lhs, rhs, has_question_mark } => Ok(Expr::Call {
                func: Callable::Static {
                    def_span: if *has_question_mark {
                        session.get_lang_item_span("fn.try_convert")
                    } else {
                        session.get_lang_item_span("fn.convert")
                    },
                    span: keyword_span.clone(),
                },
                args: vec![Expr::from_hir(lhs, session)?],
                arg_group_span: rhs.error_span_wide(),
                types: Some(Dotfish {
                    types: if *has_question_mark {
                        // `"3" as? <Int>` -> `std.convert.try_convert.<_, Int, _>("3")`
                        vec![
                            Type::Var { def_span: keyword_span.clone(), is_return: false },
                            Type::from_hir(rhs, session)?,
                            Type::Var { def_span: keyword_span.derive(SpanDeriveKind::ConvertError), is_return: false },
                        ]
                    } else {
                        // `3 as <String>` -> `std.convert.convert.<_, String>(3)`
                        vec![
                            Type::Var { def_span: keyword_span.clone(), is_return: false },
                            Type::from_hir(rhs, session)?,
                        ]
                    },
                    group_span: rhs.error_span_wide(),
                }),
                given_keyword_args: vec![],
            }),
            hir::Expr::Closure { fp, captures } => todo!(),
        }
    }

    pub fn from_ident_with_origin(id: &IdentWithOrigin, dotfish: Option<Dotfish>) -> Expr {
        match &id.origin {
            NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                NameKind::EnumVariant { parent } => {
                    return Expr::Call {
                        func: Callable::EnumInit {
                            parent_def_span: parent.clone(),
                            variant_def_span: id.def_span.clone(),
                            kind: EnumFieldKind::None,
                            span: id.span.clone(),
                        },
                        args: vec![],
                        arg_group_span: Span::None,
                        types: dotfish,
                        given_keyword_args: vec![],
                    };
                },
                _ => {},
            },
            _ => {},
        }

        Expr::Ident { id: id.clone(), dotfish }
    }

    /// If you see this value in bytecode, it's 99% likely that there's a bug in the compiler.
    pub fn dummy() -> Expr {
        Expr::Constant(Constant::dummy())
    }

    pub fn error_span_narrow(&self) -> Span {
        match self {
            Expr::Ident { id, .. } => id.span.clone(),
            Expr::Constant(c) => c.span(),
            Expr::If(r#if) => r#if.if_span.clone(),
            Expr::Match(r#match) => r#match.keyword_span.clone(),
            Expr::Block(block) => block.group_span.clone(),
            Expr::Field { fields, .. } |
            Expr::FieldUpdate { fields, .. } => merge_field_spans(fields),
            Expr::Call { func, .. } => func.error_span_narrow(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            // TODO: dotfish
            Expr::Ident { id, dotfish } => id.span.clone(),
            Expr::Constant(c) => c.span(),
            Expr::If(r#if) => r#if.if_span.merge(&r#if.true_group_span).merge(&r#if.false_group_span),
            Expr::Match(r#match) => r#match.keyword_span
                .merge(&r#match.scrutinee.error_span_wide())
                .merge(&r#match.group_span),
            Expr::Block(block) => block.group_span.clone(),
            // TODO: dotfish
            Expr::Field { lhs, fields, dotfish } => lhs.error_span_wide().merge(&merge_field_spans(fields)),
            Expr::FieldUpdate { lhs, fields, rhs } => lhs.error_span_wide()
                .merge(&merge_field_spans(fields))
                .merge(&rhs.error_span_wide()),
            Expr::Call { func, arg_group_span, .. } => func.error_span_wide().merge(arg_group_span),
        }
    }
}

pub fn true_value<S: SodigySession>(session: &S) -> Expr {
    let id = IdentWithOrigin {
        id: intern_string(b"True", session.intermediate_dir()).unwrap(),
        span: Span::None,
        origin: NameOrigin::Foreign {
            kind: NameKind::EnumVariant {
                parent: session.get_lang_item_span("type.Bool"),
            },
        },
        def_span: session.get_lang_item_span("variant.Bool.True"),
    };
    Expr::from_ident_with_origin(&id, None)
}

pub fn false_value<S: SodigySession>(session: &S) -> Expr {
    let id = IdentWithOrigin {
        id: intern_string(b"False", session.intermediate_dir()).unwrap(),
        span: Span::None,
        origin: NameOrigin::Foreign {
            kind: NameKind::EnumVariant {
                parent: session.get_lang_item_span("type.Bool"),
            },
        },
        def_span: session.get_lang_item_span("variant.Bool.False"),
    };
    Expr::from_ident_with_origin(&id, None)
}

impl Callable {
    pub fn error_span_narrow(&self) -> Span {
        match self {
            Callable::Static { span, .. } |
            Callable::StructInit { span, .. } |
            Callable::EnumInit { span, .. } |
            Callable::TupleInit { group_span: span } |
            Callable::ListInit { group_span: span } => span.clone(),
            Callable::Dynamic(expr) => expr.error_span_narrow(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Callable::Static { span, .. } |
            Callable::StructInit { span, .. } |
            Callable::EnumInit { span, .. } |
            Callable::TupleInit { group_span: span } |
            Callable::ListInit { group_span: span } => span.clone(),
            Callable::Dynamic(expr) => expr.error_span_wide(),
        }
    }
}

// TODO: can we do some optimizations here?
fn concat_strings(mut strings: Vec<Expr>, session: &Session) -> Expr {
    let def_span = session.get_lang_item_span(InfixOp::Concat.get_def_lang_item());

    match strings.len() {
        0 | 1 => unreachable!(),
        2 => {
            let rhs = strings.pop().unwrap();
            let lhs = strings.pop().unwrap();
            let derived_span = rhs.error_span_wide().derive(SpanDeriveKind::FStringConcat);

            Expr::Call {
                func: Callable::Static {
                    def_span,
                    span: derived_span.clone(),
                },
                args: vec![lhs, rhs],
                arg_group_span: derived_span.clone(),
                types: None,
                given_keyword_args: vec![],
            }
        },
        _ => {
            let tail = strings.pop().unwrap();
            let head = concat_strings(strings, session);
            let derived_span = tail.error_span_wide().derive(SpanDeriveKind::FStringConcat);

            Expr::Call {
                func: Callable::Static {
                    def_span,
                    span: derived_span.clone(),
                },
                args: vec![head, tail],
                arg_group_span: derived_span.clone(),
                types: None,
                given_keyword_args: vec![],
            }
        },
    }
}
