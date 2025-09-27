use crate::{
    Block,
    CallArg,
    Func,
    IdentWithOrigin,
    If,
    NameOrigin,
    Session,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
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
    If(If),
    Block(Block),
    Call {
        func: Box<Expr>,
        args: Vec<CallArg>,
    },
    InfixOp {
        op: InfixOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn from_ast(e: &ast::Expr, session: &mut Session) -> Result<Expr, ()> {
        match e {
            ast::Expr::Identifier { id, span } => match session.find_origin(*id) {
                Some((origin, def_span)) => {
                    if let NameOrigin::Foreign = origin {
                        session.foreign_names.insert((*id, def_span));
                    }

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
            ast::Expr::If(r#if) => Ok(Expr::If(If::from_ast(r#if, session)?)),
            ast::Expr::Block(block) => Ok(Expr::Block(Block::from_ast(block, session)?)),
            ast::Expr::Call { func, args } => {
                let func = Expr::from_ast(func, session);
                let mut new_args = Vec::with_capacity(args.len());
                let mut has_error = false;

                for arg in args.iter() {
                    match Expr::from_ast(&arg.arg, session) {
                        Ok(new_arg) => {
                            new_args.push(CallArg {
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
                    (Ok(func), false) => Ok(Expr::Call { func: Box::new(func), args: new_args }),
                    _ => Err(()),
                }
            },
            ast::Expr::Lambda { args, r#type, value, group_span } => {
                let span = group_span.begin();
                let name = name_lambda_function(span);

                // TODO
                //   1. How do I name the anonymous function?
                //   2. What do I do with the anonymous function?
                //   3. How do I register the new function to session?
                //   4. I have to identify anonymous functions, how?
                //   5. If I give a gara-name to the anonymous function, it has to be added to session.foreign_names.
                let func = ast::Func {
                    keyword_span: Span::None,
                    name,
                    name_span: span,
                    args: args.clone(),
                    r#type: r#type.as_ref().clone(),
                    value: value.as_ref().clone(),
                    attribute: ast::Attribute::new(),
                };

                match Func::from_ast(&func, session, true /* is_from_lambda */) {
                    Ok(func) => {
                        session.foreign_names.insert((name, span));
                        session.lambda_funcs.push(func);
                        Ok(Expr::Identifier(IdentWithOrigin {
                            id: name,
                            span,
                            def_span: span,
                            origin: NameOrigin::Foreign,
                        }))
                    },
                    Err(()) => Err(()),
                }
            },
            ast::Expr::InfixOp { op, lhs, rhs } => {
                match (
                    Expr::from_ast(lhs, session),
                    Expr::from_ast(rhs, session),
                ) {
                    (Ok(lhs), Ok(rhs)) => Ok(Expr::InfixOp {
                        op: *op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    _ => Err(()),
                }
            },
            _ => panic!("TODO: {e:?}"),
        }
    }
}

fn name_lambda_function(_span: Span) -> InternedString {
    // NOTE: It doesn't have to be unique because hir uses name_span and def_span to identify funcs.
    // TODO: But I want some kinda unique identifier for debugging.
    // NOTE: It has to be a short-interned-string (less than 16 characters) otherwise I have to create an intern_string_map in HirSession.
    intern_string(b"lambda_func")
}
