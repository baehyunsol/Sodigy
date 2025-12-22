use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{self as ast, Field};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};

#[derive(Clone, Debug)]
pub enum Type {
    Ident(IdentWithOrigin),

    // A type with dotted-identifiers (e.g. `std.bool.Bool`).
    // It'll eventually be lowered to `Type::Identifier`, otherwise an error.
    Path {
        id: IdentWithOrigin,
        fields: Vec<Field>,
    },

    // A type with parameters (e.g. `Result<T, U>`)
    Param {
        constructor: Box<Type>,
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
            ast::Type::Ident { id, span } => match session.find_origin_and_count_usage(*id) {
                Some((origin, def_span)) => {
                    Ok(Type::Ident(IdentWithOrigin {
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
            ast::Type::Param { constructor, args: ast_args, group_span } => {
                let mut has_error = false;
                let mut args = Vec::with_capacity(ast_args.len());
                let constructor = match Type::from_ast(constructor, session) {
                    Ok(constructor) => Some(constructor),
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
                        constructor: Box::new(constructor.unwrap()),
                        args,
                        group_span: *group_span,
                    })
                }
            },
            ast::Type::List { r#type, group_span } => {
                // TODO: I want it to be const-evaled
                let list_id = intern_string(b"List", &session.intermediate_dir).unwrap();
                let list_type = Type::Ident(IdentWithOrigin {
                    id: list_id,
                    span: *group_span,
                    origin: NameOrigin::Foreign { kind: NameKind::Struct },
                    // NOTE: It has to be session.lang_items.get("type.List"), but we don't have the lang item yet.
                    //       So we first use Prelude, then inter-hir will replace it with the lang item.
                    def_span: Span::Prelude(list_id),
                });

                Ok(Type::Param {
                    constructor: Box::new(list_type),
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
                    Ok(Type::Ident(IdentWithOrigin { def_span: Span::Prelude(f), span, .. })) => match f.try_unintern_short_string() {
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
                        spans: r#type.error_span_wide().simple_error(),
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

    pub fn error_span_narrow(&self) -> Span {
        match self {
            Type::Ident(id) => id.span,
            Type::Path { id, fields } => {
                let mut span = id.span;

                for field in fields.iter() {
                    if let Field::Name { span: field_span, .. } = field {
                        span = span.merge(*field_span);
                    }
                }

                span
            },
            Type::Param { constructor, group_span, .. } => {
                constructor.error_span_wide().merge(*group_span)
            },
            Type::Tuple { group_span, .. } => *group_span,
            Type::Func { fn_span, .. } => *fn_span,
            Type::Wildcard(span) | Type::Never(span) => *span,
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Type::Ident(id) => id.span,
            Type::Path { id, fields } => {
                let mut span = id.span;

                for field in fields.iter() {
                    if let Field::Name { span: field_span, .. } = field {
                        span = span.merge(*field_span);
                    }
                }

                span
            },
            Type::Param { constructor, group_span, .. } => {
                constructor.error_span_wide().merge(*group_span)
            },
            Type::Tuple { group_span, .. } => *group_span,
            Type::Func { fn_span, group_span, r#return, .. } => {
                let mut span = *fn_span;
                span = span.merge(*group_span);
                span.merge(r#return.error_span_wide())
            },
            Type::Wildcard(span) | Type::Never(span) => *span,
        }
    }

    // Let's say there's an alias `type OI = Option<Int>;` and a type annotation `let x: OI;`
    // In order for the alias to work, we have to replace the defspan of `OI`.
    // If something's wrong with `x`'s type, the compiler would underline the type annotation of
    // `x`, which is `OI`. So we should not change its span. Otherwise the compiler would underline
    // `Option<Int>`, which is very far from `x`.
    // We do this in the opposite way. We first clone `Option<Int>`, and replace every name and
    // span in it with `OI`'s.
    pub fn replace_name_and_span(&mut self, name: InternedString, span: Span) {
        match self {
            Type::Ident(id) => {
                id.id = name;
                id.span = span;
            },
            Type::Path { id, fields } => {
                id.id = name;
                id.span = span;
                *fields = fields.iter().map(
                    |field| match field {
                        Field::Name { name, .. } => Field::Name {
                            name: *name,
                            span: span,
                            dot_span: span,
                            is_from_alias: true,
                        },
                        _ => unreachable!(),
                    }
                ).collect();
            },
            Type::Param { constructor, args, group_span } => {
                constructor.replace_name_and_span(name, span);

                for arg in args.iter_mut() {
                    arg.replace_name_and_span(name, span);
                }

                *group_span = span;
            },
            _ => todo!(),
        }
    }
}
