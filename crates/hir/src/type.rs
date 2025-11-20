use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{self as ast, Field};
use sodigy_span::Span;
use sodigy_string::intern_string;

#[derive(Clone, Debug)]
pub enum Type {
    Identifier(IdentWithOrigin),

    // A type with dotted-identifiers (e.g. `std.bool.Bool`).
    // It'll eventually be lowered to `Type::Identifier`, otherwise an error.
    Path {
        id: IdentWithOrigin,
        fields: Vec<Field>,
    },

    // A type with parameters (e.g. `Result<T, U>`)
    Param {
        r#type: Box<Type>,
        args: Vec<Type>,
        group_span: Span,
    },
    Tuple {
        types: Vec<Type>,
        group_span: Span,
    },
    Func {
        fn_span: Span,
        group_span: Span,
        params: Vec<Type>,
        r#return: Box<Type>,
    },
    Wildcard(Span),
    Never(Span),
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
                        spans: span.simple_error(),
                        note: None,
                    });
                    Err(())
                },
            },
            ast::Type::Path { id, id_span, fields } => match session.find_origin_and_count_usage(*id) {
                Some((origin, def_span)) => {
                    Ok(Type::Path {
                        id: IdentWithOrigin {
                            id: *id,
                            span: *id_span,
                            origin,
                            def_span,
                        },
                        fields: fields.clone(),
                    })
                },
                None => {
                    session.errors.push(Error {
                        kind: ErrorKind::UndefinedName(*id),
                        spans: id_span.simple_error(),
                        note: None,
                    });
                    Err(())
                },
            },
            ast::Type::Param { r#type, args: ast_args, group_span } => {
                let mut has_error = false;
                let mut args = Vec::with_capacity(ast_args.len());
                let r#type = match Type::from_ast(r#type, session) {
                    Ok(r#type) => Some(r#type),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };

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

                if has_error {
                    Err(())
                }

                else {
                    Ok(Type::Param {
                        r#type: Box::new(r#type.unwrap()),
                        args,
                        group_span: *group_span,
                    })
                }
            },
            ast::Type::List { r#type, group_span } => {
                // TODO: I want it to be const-evaled
                let list_id = intern_string(b"List", &session.intermediate_dir).unwrap();
                let list_type = Type::Identifier(IdentWithOrigin {
                    id: list_id,
                    span: *group_span,
                    origin: NameOrigin::Foreign { kind: NameKind::Struct },
                    def_span: Span::Prelude(list_id),
                });

                Ok(Type::Param {
                    r#type: Box::new(list_type),
                    args: vec![Type::from_ast(r#type, session)?],
                    group_span: *group_span,
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
            ast::Type::Func { r#type, group_span, params: ast_params, r#return: ast_return } => {
                let mut fn_span = Span::None;
                let mut has_error = false;
                let mut has_wrong_identifier = false;
                let mut params = Vec::with_capacity(ast_params.len());

                match Type::from_ast(r#type, session) {
                    Ok(Type::Identifier(IdentWithOrigin { def_span: Span::Prelude(f), span, .. })) => match f.try_unintern_short_string() {
                        Some(f) if f == b"Fn" => {
                            fn_span = span;
                        },
                        _ => {
                            has_wrong_identifier = true;
                        },
                    },
                    Ok(_) => {
                        has_wrong_identifier = true;
                    },
                    Err(()) => {
                        has_error = true;
                    },
                }

                if has_wrong_identifier {
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidFnType,
                        spans: r#type.error_span().simple_error(),
                        note: None,
                    });
                    has_error = true;
                }

                for ast_param in ast_params.iter() {
                    match Type::from_ast(ast_param, session) {
                        Ok(param) => {
                            params.push(param);
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
                        fn_span,
                        group_span: *group_span,
                        params,
                        r#return: Box::new(r#return.unwrap()),
                    })
                }
            },
            ast::Type::Wildcard(span) => Ok(Type::Wildcard(*span)),
            ast::Type::Never(span) => Ok(Type::Never(*span)),
        }
    }

    // Error messages will use this span.
    pub fn error_span(&self) -> Span {
        match self {
            Type::Identifier(id) => id.span,
            Type::Path { id, fields } => {
                let mut span = id.span;

                for field in fields.iter() {
                    if let Field::Name { span: field_span, .. } = field {
                        span = span.merge(*field_span);
                    }
                }

                span
            },
            Type::Param { r#type, group_span, .. } => {
                r#type.error_span().merge(*group_span)
            },
            Type::Tuple { group_span, .. } => *group_span,
            Type::Func { fn_span, group_span, r#return, .. } => {
                let mut span = *fn_span;
                span = span.merge(*group_span);
                span.merge(r#return.error_span())
            },
            Type::Wildcard(span) | Type::Never(span) => *span,
        }
    }
}
