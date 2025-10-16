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
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::InfixOp;

#[derive(Clone, Debug)]
pub enum Expr {
    Identifier(IdentWithOrigin),
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
        binary: bool,
        ch: u32,
        span: Span,
    },
    If(If),
    Match(Match),
    Block(Block),
    Call {
        func: Box<Expr>,
        args: Vec<CallArg>,
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
    InfixOp {
        op: InfixOp,
        op_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn from_ast(ast_expr: &ast::Expr, session: &mut Session) -> Result<Expr, ()> {
        match ast_expr {
            ast::Expr::Identifier { id, span } => match session.find_origin_and_count_usage(*id) {
                Some((origin, def_span)) => {
                    Ok(Expr::Identifier(IdentWithOrigin {
                        id: *id,
                        span: *span,
                        origin,
                        def_span,
                    }))
                },
                None => {
                    session.errors.push(Error {
                        kind: ErrorKind::UndefinedName(*id),
                        span: *span,
                        ..Error::default()
                    });
                    Err(())
                },
            },
            ast::Expr::Number { n, span } => Ok(Expr::Number { n: *n, span: *span }),
            ast::Expr::String { binary, s, span } => Ok(Expr::String { binary: *binary, s: *s, span: *span }),
            ast::Expr::Char { binary, ch, span } => Ok(Expr::Char { binary: *binary, ch: *ch, span: *span }),
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
            ast::Expr::Lambda { args, r#type, value, group_span } => {
                let span = group_span.begin();
                let name = name_lambda_function(span, &session.intern_str_map_dir);

                let func = ast::Func {
                    keyword_span: Span::None,
                    name,
                    name_span: span,
                    generics: vec![],
                    args: args.clone(),
                    r#type: r#type.as_ref().clone(),
                    value: value.as_ref().clone(),
                    attribute: ast::Attribute::new(),
                };

                // TODO: the compiler never treats a lambda function as a top-level function...
                //       I made this choice because it's difficult to track whether it's top-level or not.
                //       I'm not sure whether it's the correct way to do this.
                match Func::from_ast(&func, session, FuncOrigin::Lambda, false /* is_top_level */) {
                    Ok(func) => {
                        session.funcs.push(func);
                        Ok(Expr::Identifier(IdentWithOrigin {
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
        }
    }
}

fn name_lambda_function(_span: Span, map_dir: &str) -> InternedString {
    // NOTE: It doesn't have to be unique because hir uses name_span and def_span to identify funcs.
    // TODO: But I want some kinda unique identifier for debugging.
    intern_string(b"lambda_func", map_dir).unwrap()
}
