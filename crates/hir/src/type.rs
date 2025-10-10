use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::intern_string;

#[derive(Clone, Debug)]
pub enum Type {
    Identifier(IdentWithOrigin),
    Generic {
        r#type: Box<Type>,
        types: Vec<Type>,
    },
    Tuple {
        types: Vec<Type>,
        group_span: Span,
    },
    Func {
        args: Vec<Type>,
        r#return: Box<Type>,
    },
}

impl Type {
    pub fn from_ast(ast_type: &ast::Type, session: &mut Session) -> Result<Type, ()> {
        match ast_type {
            ast::Type::Identifier { id, span } => match session.find_origin_and_count_usage(*id) {
                Some((origin, def_span)) => {
                    Ok(Type::Identifier(IdentWithOrigin {
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
            ast::Type::Generic { r#type, types: ast_types } => {
                let mut has_error = false;
                let mut types = Vec::with_capacity(ast_types.len());
                let r#type = match Type::from_ast(r#type, session) {
                    Ok(r#type) => Some(r#type),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };

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

                if has_error {
                    Err(())
                }

                else {
                    Ok(Type::Generic {
                        r#type: Box::new(r#type.unwrap()),
                        types,
                    })
                }
            },
            ast::Type::List { r#type, group_span } => {
                // TODO: I want it to be const-evaled
                let list_id = intern_string(b"List", &session.intern_str_map_dir).unwrap();
                let list_type = Type::Identifier(IdentWithOrigin {
                    id: list_id,
                    span: *group_span,
                    origin: NameOrigin::Foreign { kind: NameKind::Struct },
                    def_span: Span::Prelude(list_id),
                });

                Ok(Type::Generic {
                    r#type: Box::new(list_type),
                    types: vec![Type::from_ast(r#type, session)?],
                })
            },
            ast::Type::Tuple { types: ast_types, group_span } => {
                let mut has_error = false;
                let mut types = Vec::with_capacity(ast_types.len());

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

                if has_error {
                    Err(())
                }

                else {
                    Ok(Type::Tuple {
                        types,
                        group_span: *group_span,
                    })
                }
            },
            ast::Type::Func { r#type, args: ast_args, r#return: ast_return } => {
                let mut has_error = false;
                let mut has_wrong_identifier = false;
                let mut args = Vec::with_capacity(ast_args.len());

                match Type::from_ast(r#type, session) {
                    Ok(func) => match func {
                        Type::Identifier(id) => match id.def_span {
                            Span::Prelude(f) => match f.try_unintern_short_string() {
                                Some(f) if f == b"Fn" => {},
                                _ => {
                                    has_wrong_identifier = true;
                                },
                            },
                            _ => {
                                has_wrong_identifier = true;
                            },
                        },
                        _ => {
                            has_wrong_identifier = true;
                        },
                    },
                    Err(()) => {
                        has_error = true;
                    },
                }

                if has_wrong_identifier {
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidFnType,
                        span: r#type.error_span(),
                        ..Error::default()
                    });
                    has_error = true;
                }

                for ast_arg in ast_args.iter() {
                    match Type::from_ast(ast_arg, session) {
                        Ok(arg) => {
                            args.push(arg);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                let r#return = match Type::from_ast(ast_return, session) {
                    Ok(r#return) => Some(r#return),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };

                if has_error {
                    Err(())
                }

                else {
                    Ok(Type::Func {
                        args,
                        r#return: Box::new(r#return.unwrap()),
                    })
                }
            },
            _ => panic!("TODO: {ast_type:?}"),
        }
    }
}
