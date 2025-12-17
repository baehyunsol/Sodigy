use crate::{
    Block,
    CallArg,
    Func,
    FuncOrigin,
    Match,
    If,
    Session,
    StructInitField,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_parse::{self as ast, Field};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, intern_string, unintern_string};
use sodigy_token::{InfixOp, PostfixOp, PrefixOp};

mod pipeline;
use pipeline::replace_dollar;

#[derive(Clone, Debug)]
pub enum Expr {
    Ident(IdentWithOrigin),
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
        r#struct: Box<Expr>,
        fields: Vec<StructInitField>,
        group_span: Span,
    },
    // `a.b.c.d` is lowered to `Path { lhs: a, fields: [b, c, d] }`
    Path {
        lhs: Box<Expr>,
        fields: Vec<Field>,
    },
    FieldModifier {
        fields: Vec<(InternedString, Span)>,
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
    pub fn from_ast(ast_expr: &ast::Expr, session: &mut Session) -> Result<Expr, ()> {
        match ast_expr {
            ast::Expr::Ident { id, span } => match session.find_origin_and_count_usage(*id) {
                Some((origin, def_span)) => {
                    Ok(Expr::Ident(IdentWithOrigin {
                        id: *id,
                        span: *span,
                        origin,
                        def_span,
                    }))
                },
                None => {
                    session.errors.push(Error {
                        kind: ErrorKind::UndefinedName(*id),
                        spans: span.simple_error(),
                        note: None,
                    });
                    Err(())
                },
            },
            ast::Expr::Number { n, span } => Ok(Expr::Number { n: n.clone(), span: *span }),
            ast::Expr::String { binary, s, span } => Ok(Expr::String { binary: *binary, s: *s, span: *span }),
            ast::Expr::Char { ch, span } => Ok(Expr::Char { ch: *ch, span: *span }),
            ast::Expr::Byte { b, span } => Ok(Expr::Byte { b: *b, span: *span }),
            ast::Expr::If(r#if) => Ok(Expr::If(If::from_ast(r#if, session)?)),
            ast::Expr::Match(r#match) => Ok(Expr::Match(Match::from_ast(r#match, session)?)),
            ast::Expr::Block(block) => Ok(Expr::Block(Block::from_ast(block, session, false /* is_top_level */)?)),
            ast::Expr::Call { func, args } => {
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
                    (Ok(func), false) => Ok(Expr::Call { func: Box::new(func), args: hir_args }),
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
                        ast::ExprOrString::String(s) => {
                            elements.push(ExprOrString::String(*s));
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
            ast::Expr::StructInit { r#struct, fields, group_span } => {
                let r#struct = Expr::from_ast(r#struct, session);
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

                match (r#struct, has_error) {
                    (Ok(r#struct), false) => Ok(Expr::StructInit {
                        r#struct: Box::new(r#struct),
                        fields: hir_fields,
                        group_span: *group_span,
                    }),
                    _ => Err(()),
                }
            },
            ast::Expr::Path { lhs, field } => match Expr::from_ast(lhs, session) {
                Ok(Expr::Path { lhs, mut fields }) => {
                    fields.push(*field);
                    Ok(Expr::Path {
                        lhs,
                        fields,
                    })
                },
                Ok(lhs) => Ok(Expr::Path {
                    lhs: Box::new(lhs),
                    fields: vec![*field],
                }),
                Err(()) => Err(()),
            },
            ast::Expr::FieldModifier { fields, lhs, rhs } => match (
                Expr::from_ast(lhs, session),
                Expr::from_ast(rhs, session),
            ) {
                (Ok(lhs), Ok(rhs)) => Ok(Expr::FieldModifier {
                    fields: fields.clone(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                _ => Err(()),
            },
            ast::Expr::Lambda { params, r#type, value, group_span } => {
                let span = group_span.begin();
                let name = name_lambda_function(span, &session.intermediate_dir);

                let func = ast::Func {
                    keyword_span: Span::None,
                    name,
                    name_span: span,
                    generics: vec![],
                    generic_group_span: None,
                    params: params.clone(),
                    r#type: r#type.as_ref().clone(),
                    value: Some(value.as_ref().clone()),
                    attribute: ast::Attribute::new(),
                };

                // TODO: the compiler never treats a lambda function as a top-level function...
                //       I made this choice because it's difficult to track whether it's top-level or not.
                //       I'm not sure whether it's the correct way to do this.
                match Func::from_ast(&func, session, FuncOrigin::Lambda, false /* is_top_level */) {
                    Ok(func) => {
                        session.funcs.push(func);
                        Ok(Expr::Ident(IdentWithOrigin {
                            id: name,
                            span,
                            def_span: span,
                            origin: NameOrigin::Foreign {
                                kind: NameKind::Func,
                            },
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
                            ast::Expr::Ident { id, .. } => format!(
                                "Unlike gleam or bash, you have to explicitly pipe the value. Try `{}($)` instead of `{}`",
                                String::from_utf8_lossy(&unintern_string(*id, &session.intermediate_dir).unwrap().unwrap()),
                                String::from_utf8_lossy(&unintern_string(*id, &session.intermediate_dir).unwrap().unwrap()),
                            ),
                            ast::Expr::Path { lhs, field } => match try_render_dotted_name(lhs) {
                                Some(name) => format!("Unlike gleam or bash, you have to explicitly use the value. Try `{name}($)` instead of `{name}`"),
                                None => String::from("There's no `$` here."),
                            },
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
                                    span: value.error_span(),
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
                    group_span: Span::None,
                    lets: values[..(values.len() - 1)].iter().enumerate().map(
                        |(i, value)| ast::Let {
                            keyword_span: Span::None,
                            name: intern_string(format!("{}{i}", "$".repeat(session.nested_pipeline_depth)).as_bytes(), &session.intermediate_dir).unwrap(),
                            name_span: pipe_spans[i],
                            r#type: None,
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

                let lowered_block = Block::from_ast(&ast_block, session, false /* is_top_level */);
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

    pub fn error_span(&self) -> Span {
        match self {
            Expr::Ident(IdentWithOrigin { span, .. }) |
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } => *span,
            Expr::Path { lhs, fields } => {
                let mut span = lhs.error_span();

                for field in fields.iter() {
                    span = span.merge(field.unwrap_span());
                }

                span
            },
            _ => todo!(),
        }
    }
}

fn name_lambda_function(_span: Span, map_dir: &str) -> InternedString {
    // NOTE: It doesn't have to be unique because hir uses name_span and def_span to identify funcs.
    // TODO: But I want some kinda unique identifier for debugging.
    intern_string(b"lambda_func", map_dir).unwrap()
}

fn try_render_dotted_name(expr: &ast::Expr) -> Option<String> {
    todo!()
}

#[derive(Clone, Debug)]
pub enum ExprOrString {
    Expr(Expr),
    String(InternedString),
}
