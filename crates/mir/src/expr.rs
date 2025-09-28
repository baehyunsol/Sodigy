use crate::{Block, If, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_hir as hir;
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_span::Span;

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
        func: Callable,
        args: Vec<Expr>,

        // We have to do tail-call analysis after function inlining!
        // tail_call: bool,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum Callable {
    // There must be `HashMap<Span, Func>` somewhere
    Static(Span),
}

impl Expr {
    pub fn from_hir(hir_expr: &hir::Expr, session: &mut Session) -> Result<Expr, ()> {
        match hir_expr {
            hir::Expr::Identifier(id) => Ok(Expr::Identifier(*id)),
            hir::Expr::Number { n, span } => Ok(Expr::Number {
                n: *n,
                span: *span,
            }),
            hir::Expr::If(r#if) => todo!(),
            hir::Expr::Block(block) => match Block::from_hir(block, session) {
                Ok(block) => Ok(Expr::Block(block)),
                Err(_) => Err(()),
            },
            hir::Expr::Call {
                func,
                args,
            } => {
                let mut has_error = false;
                let mut def_span = None;

                let func = match Expr::from_hir(func, session) {
                    Ok(Expr::Identifier(id)) => match id.origin {
                        NameOrigin::Local { kind } |
                        NameOrigin::Foreign { kind } => match kind {
                            NameKind::Func => {
                                def_span = Some(id.def_span);
                                Callable::Static(id.def_span)
                            },
                            _ => todo!(),
                        },
                        NameOrigin::FuncArg { .. } => todo!(),
                    },
                    Ok(id) => todo!(),
                    Err(_) => {
                        has_error = true;
                        todo!()
                    },
                };

                let args = match def_span {
                    Some(def_span) => match session.func_args.get(&def_span) {
                        Some(arg_defs) => {
                            let arg_defs = arg_defs.to_vec();
                            let mut mir_args: Vec<Option<Expr>> = vec![None; arg_defs.len()];

                            // used for error messages
                            let mut keyword_spans = vec![None; arg_defs.len()];

                            // Positional args cannot come after a keyword arg, and hir guarantees that.
                            let mut positional_arg_cursor = 0;

                            for arg in args.iter() {
                                match arg.keyword {
                                    Some((keyword, keyword_span)) => {
                                        let mut arg_index = None;

                                        for (i, arg_def) in arg_defs.iter().enumerate() {
                                            if arg_def.name == keyword {
                                                arg_index = Some(i);
                                                break;
                                            }
                                        }

                                        match arg_index {
                                            Some(i) => {
                                                if let Some(mir_arg) = &mir_args[i] {
                                                    session.errors.push(Error {
                                                        kind: ErrorKind::KeywordArgumentRepeated(keyword),
                                                        span: keyword_span,
                                                        extra_span: if let Some(span) = &keyword_spans[i] {
                                                            Some(*span)
                                                        } else {
                                                            Some(mir_arg.error_span())
                                                        },
                                                        ..Error::default()
                                                    });
                                                }

                                                match Expr::from_hir(&arg.arg, session) {
                                                    Ok(arg) => {
                                                        mir_args[i] = Some(arg);
                                                    },
                                                    Err(_) => {
                                                        has_error = true;
                                                    },
                                                }

                                                keyword_spans[i] = Some(keyword_span)
                                            },
                                            None => {
                                                session.errors.push(Error {
                                                    kind: ErrorKind::InvalidKeywordArgument(keyword),
                                                    span: keyword_span,
                                                    ..Error::default()
                                                });
                                                has_error = true;
                                            },
                                        }
                                    },
                                    None => {
                                        match Expr::from_hir(&arg.arg, session) {
                                            Ok(arg) => {
                                                mir_args[positional_arg_cursor] = Some(arg);
                                            },
                                            Err(_) => {
                                                has_error = true;
                                            },
                                        }

                                        positional_arg_cursor += 1;
                                    },
                                }
                            }

                            // TODO: if any of `mir_args` is None, it's an error
                            //       but I have to come up with a nice way to generate helpful error messages
                            mir_args.into_iter().map(|arg| arg.unwrap()).collect()
                        },
                        None => todo!(),
                    },
                    None => todo!(),
                };

                if has_error {
                    Err(())
                }

                else {
                    Ok(Expr::Call { func, args })
                }
            },

            // TODO: these have to be `mir::Expr::Call`, but how?
            hir::Expr::Tuple { elements, .. } => todo!(),
            hir::Expr::List { elements, .. } => todo!(),
            hir::Expr::InfixOp { op, lhs, rhs } => todo!(),
        }
    }

    // span for error messages
    pub fn error_span(&self) -> Span {
        match self {
            Expr::Identifier(id) => id.span,
            Expr::Number { span, .. } => *span,
            Expr::If(r#if) => r#if.keyword_span,
            Expr::Block(block) => block.group_span,
            Expr::Call { func, .. } => match func {
                Callable::Static(span) => *span,
            },
        }
    }
}
