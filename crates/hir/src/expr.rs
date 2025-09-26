use crate::{Block, CallArg, If, NameOrigin, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::InfixOp;

#[derive(Clone, Debug)]
pub enum Expr {
    Identifier {
        id: InternedString,
        span: Span,
        origin: NameOrigin,

        // It's used to uniquely identify the identifiers.
        def_span: Span,
    },
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

                    Ok(Expr::Identifier {
                        id: *id,
                        span: *span,
                        origin,
                        def_span,
                    })
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
