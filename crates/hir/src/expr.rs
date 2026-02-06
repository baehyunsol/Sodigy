use crate::{
    Block,
    CallArg,
    Func,
    FuncOrigin,
    If,
    Match,
    Path,
    Session,
    StructInitField,
    Type,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_parse::{self as ast, Field, merge_field_spans};
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{InfixOp, PostfixOp, PrefixOp};

mod pipeline;
use pipeline::replace_dollar;

#[derive(Clone, Debug)]
pub enum Expr {
    // We are not sure whether `a.b.c` is `Expr::Path` or `Expr::Field`.
    // We don't know that until inter-hir. So we just lower `a.b.c` to
    // `Expr::Path`, and inter-hir will resolve it later.
    Path(Path),
    Number {
        n: InternedNumber,
        span: Span,
    },
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
    Match(Match),
    Block(Block),
    Call {
        func: Box<Expr>,
        args: Vec<CallArg>,
        arg_group_span: Span,
    },
    FormattedString {
        raw: bool,
        elements: Vec<ExprOrString>,
        span: Span,
    },
    Tuple {
        elements: Vec<Expr>,
        group_span: Span,
    },
    List {
        elements: Vec<Expr>,
        group_span: Span,
    },
    StructInit {
        constructor: Path,
        fields: Vec<StructInitField>,
        group_span: Span,
    },
    // `a.b.c.d` is lowered to `Path(Path { id: a, fields: [b, c, d] })`.
    // `(1 + 1).a.b.c` is lowered to `Field { lhs: 1 + 1, fields: [b, c, d] }`.
    //
    // `a.b.c.d` can be a field or a path, but hir cannot know that. Inter-hir will
    // figure that out and resolve it.
    Field {
        lhs: Box<Expr>,
        fields: Vec<Field>,

        // dotfish operators
        types: Vec<Option<Vec<Type>>>,
    },
    FieldUpdate {
        fields: Vec<Field>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        op_span: Span,
        rhs: Box<Expr>,
    },
    InfixOp {
        op: InfixOp,
        op_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    PostfixOp {
        op: PostfixOp,
        op_span: Span,
        lhs: Box<Expr>,
    },
}

impl Expr {
    pub fn dummy() -> Self {
        // You shouldn't see this value in bytecodes.
        // I put a random-looking number for easier debugging.
        Expr::Char { ch: 49773, span: Span::None }
    }

    pub fn from_ast(ast_expr: &ast::Expr, session: &mut Session) -> Result<Expr, ()> {
        match ast_expr {
            ast::Expr::Path(p) => Ok(Expr::Path(Path::from_ast(p, session)?)),
            ast::Expr::Number { n, span } => Ok(Expr::Number { n: n.clone(), span: *span }),
            ast::Expr::String { binary, s, span } => Ok(Expr::String { binary: *binary, s: *s, span: *span }),
            ast::Expr::Char { ch, span } => Ok(Expr::Char { ch: *ch, span: *span }),
            ast::Expr::Byte { b, span } => Ok(Expr::Byte { b: *b, span: *span }),
            ast::Expr::If(r#if) => Ok(Expr::If(If::from_ast(r#if, session)?)),
            ast::Expr::Match(r#match) => Ok(Expr::Match(Match::from_ast(r#match, session)?)),
            ast::Expr::Block(block) => Ok(Expr::Block(Block::from_ast(block, session)?)),
            ast::Expr::Call { func, args, arg_group_span } => {
                let func = Expr::from_ast(func, session);
                let mut hir_args = Vec::with_capacity(args.len());
                let mut has_error = false;

                for arg in args.iter() {
                    match Expr::from_ast(&arg.arg, session) {
                        Ok(new_arg) => {
                            hir_args.push(CallArg {
                                keyword: arg.keyword,
                                arg: new_arg,
                            });
                        },
                        Err(_) => {
                            has_error = true;
                        },
                    }
                }

                match (func, has_error) {
                    (Ok(func), false) => Ok(Expr::Call {
                        func: Box::new(func),
                        args: hir_args,
                        arg_group_span: *arg_group_span,
                    }),
                    _ => Err(()),
                }
            },
            ast::Expr::FormattedString { raw, elements: ast_elements, span } => {
                let mut has_error = false;
                let mut elements = Vec::with_capacity(ast_elements.len());

                for ast_element in ast_elements.iter() {
                    match ast_element {
                        ast::ExprOrString::Expr(e) => match Expr::from_ast(e, session) {
                            Ok(e) => {
                                elements.push(ExprOrString::Expr(e));
                            },
                            Err(()) => {
                                has_error = true;
                            },
                        },
                        ast::ExprOrString::String { s, span } => {
                            elements.push(ExprOrString::String { s: *s, span: *span });
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(Expr::FormattedString {
                        raw: *raw,
                        elements,
                        span: *span,
                    })
                }
            },
            ast::Expr::Tuple { elements, group_span } |
            ast::Expr::List { elements, group_span } => {
                let is_tuple = matches!(ast_expr, ast::Expr::Tuple { .. });
                let group_span = *group_span;
                let mut has_error = false;
                let mut new_elements = Vec::with_capacity(elements.len());

                for element in elements.iter() {
                    match Expr::from_ast(element, session) {
                        Ok(element) => {
                            new_elements.push(element);
                        },
                        Err(_) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else if is_tuple {
                    Ok(Expr::Tuple {
                        elements: new_elements,
                        group_span,
                    })
                }

                else {
                    Ok(Expr::List {
                        elements: new_elements,
                        group_span,
                    })
                }
            },
            ast::Expr::StructInit { constructor, fields, group_span } => {
                let constructor = Path::from_ast(constructor, session);
                let mut hir_fields = Vec::with_capacity(fields.len());
                let mut has_error = false;

                for field in fields.iter() {
                    match Expr::from_ast(&field.value, session) {
                        Ok(value) => {
                            hir_fields.push(StructInitField {
                                name: field.name,
                                name_span: field.name_span,
                                value,
                            });
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                match (constructor, has_error) {
                    (Ok(constructor), false) => Ok(Expr::StructInit {
                        constructor,
                        fields: hir_fields,
                        group_span: *group_span,
                    }),
                    _ => Err(()),
                }
            },
            ast::Expr::Field { lhs, field, r#type: ast_type } => {
                let mut has_error = false;
                let dotfish = match ast_type {
                    Some(ast_types) => {
                        let mut types = vec![];

                        for ast_type in ast_types.iter() {
                            match Type::from_ast(ast_type, session) {
                                Ok(r#type) => {
                                    types.push(r#type);
                                },
                                Err(()) => {
                                    has_error = true;
                                },
                            }
                        }

                        Some(types)
                    },
                    None => None,
                };

                match Expr::from_ast(lhs, session) {
                    _ if has_error => Err(()),
                    Ok(Expr::Field { lhs, mut fields, mut types }) => {
                        fields.push(*field);
                        types.push(dotfish);
                        Ok(Expr::Field {
                            lhs,
                            fields,
                            types,
                        })
                    },
                    Ok(lhs) => Ok(Expr::Field {
                        lhs: Box::new(lhs),
                        fields: vec![*field],
                        types: vec![None, dotfish],
                    }),
                    Err(()) => Err(()),
                }
            },
            ast::Expr::FieldUpdate { fields, lhs, rhs } => match (
                Expr::from_ast(lhs, session),
                Expr::from_ast(rhs, session),
            ) {
                (Ok(lhs), Ok(rhs)) => Ok(Expr::FieldUpdate {
                    fields: fields.clone(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                _ => Err(()),
            },
            ast::Expr::Lambda(ast::Lambda {
                is_pure,
                impure_keyword_span,
                params,
                param_group_span,
                type_annot,
                value,
                ..
            }) => {
                let span = param_group_span.begin().derive(SpanDeriveKind::Lambda);
                let name = name_lambda_function(span, &session.intermediate_dir);

                let func = ast::Func {
                    is_pure: *is_pure,
                    impure_keyword_span: *impure_keyword_span,
                    keyword_span: span,
                    name,
                    name_span: span,
                    generics: vec![],
                    generic_group_span: None,
                    params: params.clone(),
                    type_annot: type_annot.as_ref().clone(),
                    value: Some(value.as_ref().clone()),
                    attribute: ast::Attribute::new(),
                };

                // TODO: the compiler never treats a lambda function as a top-level function...
                //       I made this choice because it's difficult to track whether it's top-level or not.
                //       I'm not sure whether it's the correct way to do this.
                match Func::from_ast(&func, session, FuncOrigin::Lambda) {
                    Ok(func) => {
                        session.push_lambda(func);
                        Ok(Expr::Path(Path {
                            id: IdentWithOrigin {
                                id: name,
                                span,
                                def_span: span,
                                origin: NameOrigin::Foreign {
                                    kind: NameKind::Func,
                                },
                            },
                            fields: vec![],
                            types: vec![None],
                        }))
                    },
                    Err(()) => Err(()),
                }
            },
            ast::Expr::PrefixOp { op, op_span, rhs } => Ok(Expr::PrefixOp {
                op: *op,
                op_span: *op_span,
                rhs: Box::new(Expr::from_ast(rhs, session)?),
            }),
            ast::Expr::InfixOp { op, op_span, lhs, rhs } => {
                match (
                    Expr::from_ast(lhs, session),
                    Expr::from_ast(rhs, session),
                ) {
                    (Ok(lhs), Ok(rhs)) => Ok(Expr::InfixOp {
                        op: *op,
                        op_span: *op_span,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    _ => Err(()),
                }
            },
            ast::Expr::PostfixOp { op, op_span, lhs } => Ok(Expr::PostfixOp {
                op: *op,
                op_span: *op_span,
                lhs: Box::new(Expr::from_ast(lhs, session)?),
            }),
            // `a() |> b($) + (c() |> d($)) |> e($);`
            // ->
            // `{ let $0 = a(); let $1 = b($0) + { let $$0 = c(); d($$0) }; e($1) }`
            ast::Expr::Pipeline { values, pipe_spans } => {
                session.nested_pipeline_depth += 1;
                let mut values = values.to_vec();
                let mut has_error = false;

                for (i, value) in values.iter_mut().skip(1).enumerate() {
                    let ident = intern_string(format!("{}{i}", "$".repeat(session.nested_pipeline_depth)).as_bytes(), &session.intermediate_dir).unwrap();
                    let mut replaced_spans = vec![];
                    let mut has_nested_pipeline = false;

                    replace_dollar(
                        value,
                        ident,
                        &mut replaced_spans,
                        &mut has_nested_pipeline,
                    );

                    if replaced_spans.is_empty() {
                        let value_note = match value {
                            ast::Expr::Path(p) => format!(
                                "Unlike gleam or bash, you have to explicitly pipe the value. Try `{}($)` instead of `{}`",
                                p.unintern_or_default(&session.intermediate_dir),
                                p.unintern_or_default(&session.intermediate_dir),
                            ),
                            _ if has_nested_pipeline => String::from("I see a nested pipeline in this expression. `$` always captures the value of the closest (inner-most) pipeline. Perhaps you have to use a block expression to explicitly give names to values."),
                            _ => String::from("There's no `$` here."),
                        };

                        session.errors.push(Error {
                            kind: ErrorKind::DisconnectedPipeline,
                            spans: vec![
                                RenderableSpan {
                                    span: pipe_spans[i],
                                    auxiliary: false,
                                    note: Some(String::from("It pipes a value, but no one uses the value.")),
                                },
                                RenderableSpan {
                                    span: value.error_span_wide(),
                                    auxiliary: true,
                                    note: Some(value_note),
                                },
                            ],
                            note: None,
                        });
                        has_error = true;
                    }
                }

                let ast_block = ast::Block {
                    group_span: pipe_spans[0].derive(SpanDeriveKind::Pipeline),
                    lets: values[..(values.len() - 1)].iter().enumerate().map(
                        |(i, value)| ast::Let {
                            keyword_span: pipe_spans[i].derive(SpanDeriveKind::Pipeline),
                            name: intern_string(format!("{}{i}", "$".repeat(session.nested_pipeline_depth)).as_bytes(), &session.intermediate_dir).unwrap(),
                            name_span: pipe_spans[i].derive(SpanDeriveKind::Pipeline),
                            type_annot: None,
                            value: value.clone(),
                            attribute: ast::Attribute::new(),
                            from_pipeline: true,
                        }
                    ).collect(),
                    funcs: vec![],
                    structs: vec![],
                    enums: vec![],
                    asserts: vec![],
                    aliases: vec![],
                    uses: vec![],
                    modules: vec![],
                    value: Box::new(values.last().map(|v| v.clone())),
                    attribute: None,
                    from_pipeline: true,
                };

                let lowered_block = Block::from_ast(&ast_block, session);
                session.nested_pipeline_depth -= 1;
                let lowered_block = lowered_block?;

                if has_error {
                    Err(())
                }

                else {
                    Ok(Expr::Block(lowered_block))
                }
            },

            // If it belongs to a pipeline, it must already be lowered to a `hir::Expr::Ident`.
            ast::Expr::PipelineData(span) => {
                session.errors.push(Error {
                    kind: ErrorKind::DollarOutsidePipeline,
                    spans: span.simple_error(),
                    note: None,
                });
                Err(())
            },
        }
    }

    pub fn error_span_narrow(&self) -> Span {
        match self {
            Expr::Path(p) => p.error_span_narrow(),
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } |
            Expr::FormattedString { span, .. } => *span,
            Expr::If(r#if) => r#if.if_span,
            Expr::Match(r#match) => r#match.keyword_span,
            Expr::Block(block) => block.group_span,
            Expr::Call { func, .. } => func.error_span_narrow(),
            Expr::Tuple { group_span, .. } |
            Expr::List { group_span, .. } => *group_span,
            Expr::StructInit { constructor, .. } => constructor.error_span_narrow(),
            Expr::Field { fields, .. } |
            Expr::FieldUpdate { fields, .. } => merge_field_spans(fields),
            Expr::PrefixOp { op_span, .. } |
            Expr::InfixOp { op_span, .. } |
            Expr::PostfixOp { op_span, .. } => *op_span,
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Expr::Path(p) => p.error_span_wide(),
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } |
            Expr::FormattedString { span, .. } => *span,
            Expr::If(r#if) => r#if.if_span
                .merge(r#if.cond.error_span_wide())
                .merge(r#if.else_span)
                .merge(r#if.true_group_span)
                .merge(r#if.false_group_span),
            Expr::Match(r#match) => r#match.keyword_span.merge(r#match.group_span),
            Expr::Block(block) => block.group_span,
            Expr::Call { func, arg_group_span, .. } => func.error_span_wide().merge(*arg_group_span),
            Expr::Tuple { group_span, .. } |
            Expr::List { group_span, .. } => *group_span,
            Expr::StructInit { constructor, group_span, .. } => constructor.error_span_wide().merge(*group_span),

            // TODO: dump dotfish
            Expr::Field { lhs, fields, types } => lhs.error_span_wide().merge(merge_field_spans(fields)),
            Expr::FieldUpdate { lhs, fields, rhs } => lhs.error_span_wide()
                .merge(merge_field_spans(fields))
                .merge(rhs.error_span_wide()),
            Expr::PrefixOp { op_span, rhs, .. } => op_span.merge(rhs.error_span_wide()),
            Expr::InfixOp { lhs, op_span, rhs, .. } => lhs.error_span_wide().merge(*op_span).merge(rhs.error_span_wide()),
            Expr::PostfixOp { op_span, lhs, .. } => lhs.error_span_wide().merge(*op_span),
        }
    }
}

fn name_lambda_function(_span: Span, map_dir: &str) -> InternedString {
    // NOTE: It doesn't have to be unique because hir uses name_span and def_span to identify funcs.
    // TODO: But I want some kinda unique identifier for debugging.
    intern_string(b"lambda_func", map_dir).unwrap()
}

#[derive(Clone, Debug)]
pub enum ExprOrString {
    Expr(Expr),
    String { s: InternedString, span: Span },
}
