use crate::{Block, If, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_hir as hir;
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::InfixOp;

#[derive(Clone, Debug)]
pub enum Expr {
    Identifier(IdentWithOrigin),
    Number {
        n: InternedNumber,
        span: Span,
    },
    // Ideally, we can create `Callable::StringInit`, but that'd struggle with long strings.
    String {
        binary: bool,
        s: InternedString,
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

#[derive(Clone, Debug)]
pub enum Callable {
    // There must be `HashMap<Span, Func>` somewhere
    Static {
        def_span: Span,
        span: Span,
    },

    // It's a functor and can only be evaluated at runtime.
    Dynamic(Box<Expr>),

    // Infix operations before type inference. For example, `+` in `3 + 4` is first
    // lowered to a generic-addition, then after the compiler finds out that the both operands are
    // integer, it's lowered to integer-addition.
    GenericInfixOp {
        op: InfixOp,
        span: Span,
    },

    ListInit {
        group_span: Span,
    },
}

impl Expr {
    pub fn from_hir(hir_expr: &hir::Expr, session: &mut Session) -> Result<Expr, ()> {
        match hir_expr {
            hir::Expr::Identifier(id) => Ok(Expr::Identifier(*id)),
            hir::Expr::Number { n, span } => Ok(Expr::Number {
                n: *n,
                span: *span,
            }),
            hir::Expr::String { binary, s, span } => Ok(Expr::String {
                binary: *binary,
                s: *s,
                span: *span,
            }),
            hir::Expr::If(r#if) => match If::from_hir(r#if, session) {
                Ok(r#if) => Ok(Expr::If(r#if)),
                Err(()) => Err(()),
            },
            hir::Expr::Block(block) => match Block::from_hir(block, session) {
                Ok(block) => Ok(Expr::Block(block)),
                Err(()) => Err(()),
            },
            hir::Expr::Call {
                func,
                args: hir_args,
            } => {
                let mut has_error = false;
                let mut def_span = None;

                let func = match Expr::from_hir(func, session) {
                    Ok(Expr::Identifier(id)) => match id.origin {
                        NameOrigin::Local { kind } |
                        NameOrigin::Foreign { kind } => match kind {
                            NameKind::Func => {
                                def_span = Some(id.def_span);
                                Callable::Static {
                                    def_span: id.def_span,
                                    span: id.span,
                                }
                            },
                            // The programmer defines a functor using `let` keyword
                            // and calling it. In this case, we have to dynamically call the
                            // function on runtime. (Maybe we can do some optimizations and turn it into a static call?)
                            NameKind::Let => {
                                def_span = Some(id.def_span);
                                Callable::Dynamic(Box::new(Expr::Identifier(id)))
                            },
                            _ => panic!("TODO: {kind:?}"),
                        },
                        NameOrigin::FuncArg { .. } => todo!(),
                    },
                    Ok(id) => todo!(),
                    Err(()) => {
                        has_error = true;
                        todo!()
                    },
                };

                // If we know `def_span` and the `def_span` is in `func_shapes`,
                // we know the exact definition of the function, and can process keyword arguments and default values.
                let mut mir_args = match def_span {
                    Some(def_span) => match session.func_shapes.get(&def_span) {
                        Some(arg_defs) => {
                            let arg_defs = arg_defs.to_vec();
                            let mut mir_args: Vec<Option<Expr>> = vec![None; arg_defs.len()];

                            // used for error messages
                            let mut keyword_spans = vec![None; arg_defs.len()];

                            // Positional args cannot come after a keyword arg, and hir guarantees that.
                            let mut positional_arg_cursor = 0;

                            for hir_arg in hir_args.iter() {
                                match hir_arg.keyword {
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

                                                match Expr::from_hir(&hir_arg.arg, session) {
                                                    Ok(arg) => {
                                                        mir_args[i] = Some(arg);
                                                    },
                                                    Err(()) => {
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

                            for i in 0..arg_defs.len() {
                                match (&mir_args[i], &arg_defs[i].default_value) {
                                    (None, Some(default_value)) => {
                                        mir_args[i] = Some(Expr::Identifier(*default_value));
                                    },
                                    _ => {},
                                }
                            }

                            let mut result = Vec::with_capacity(mir_args.len());

                            for mir_arg in mir_args.into_iter() {
                                if let Some(mir_arg) = mir_arg {
                                    result.push(mir_arg);
                                }

                                // If mir_arg is None, that's a compile error, but we're not raising an error yet.
                                // We'll raise an error after type-check/inference, so that we can add more information to the error message.
                            }

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
                                        span: keyword_span,
                                        ..Error::default()
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
                    Ok(Expr::Call { func, args })
                }
            },

            hir::Expr::Tuple { elements, .. } => todo!(),
            hir::Expr::List { elements, group_span } => {
                let mut mir_elements = Vec::with_capacity(elements.len());
                let mut has_error = false;

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
                    Ok(Expr::Call {
                        func: Callable::ListInit {
                            group_span: *group_span,
                        },
                        args: mir_elements,
                    })
                }
            },
            hir::Expr::StructInit { r#struct, fields, group_span } => todo!(),
            hir::Expr::Path { lhs, fields } => todo!(),
            hir::Expr::InfixOp { op, op_span, lhs, rhs } => {
                match (
                    Expr::from_hir(lhs, session),
                    Expr::from_hir(rhs, session),
                ) {
                    (Ok(lhs), Ok(rhs)) => Ok(Expr::Call {
                        func: Callable::GenericInfixOp {
                            op: *op,
                            span: *op_span,
                        },
                        args: vec![lhs, rhs],
                    }),
                    _ => Err(()),
                }
            },
        }
    }

    // span for error messages
    pub fn error_span(&self) -> Span {
        match self {
            Expr::Identifier(id) => id.span,
            Expr::Number { span, .. } |
            Expr::String { span, .. } => *span,
            Expr::If(r#if) => r#if.if_span,
            Expr::Block(block) => block.group_span,
            Expr::Call { func, .. } => match func {
                Callable::Static { span, .. } |
                Callable::GenericInfixOp { span, .. } => *span,
                Callable::Dynamic(expr) => expr.error_span(),
                Callable::ListInit { group_span, .. } => *group_span,
            },
        }
    }
}
