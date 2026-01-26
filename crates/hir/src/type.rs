use crate::{Path, Session};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{self as ast};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};

#[derive(Clone, Debug)]
pub enum Type {
    Path(Path),

    // A type with parameters (e.g. `Result<T, U>`)
    Param {
        constructor: Path,
        args: Vec<Type>,
        group_span: Span,
    },
    Tuple {
        types: Vec<Type>,
        group_span: Span,
    },
    Func {
        fn_constructor: Path,
        group_span: Span,
        params: Vec<Type>,
        r#return: Box<Type>,
    },
    Wildcard(Span),
    Never(Span),
}

#[derive(Clone, Debug)]
pub struct TypeAssertion {
    pub name_span: Span,
    pub type_span: Span,
    pub r#type: Type,
}

impl Type {
    pub fn from_ast(ast_type: &ast::Type, session: &mut Session) -> Result<Type, ()> {
        match ast_type {
            ast::Type::Path(p) => Ok(Type::Path(Path::from_ast(p, session)?)),
            ast::Type::Param { constructor, args: ast_args, group_span } => {
                let mut has_error = false;
                let mut args = Vec::with_capacity(ast_args.len());
                let constructor = match Path::from_ast(constructor, session) {
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
                        constructor: constructor.unwrap(),
                        args,
                        group_span: *group_span,
                    })
                }
            },
            ast::Type::List { r#type, group_span } => {
                // TODO: I want `list_id` to be const-evaled
                let list_id = intern_string(b"List", &session.intermediate_dir).unwrap();

                Ok(Type::Param {
                    constructor: Path {
                        id: IdentWithOrigin {
                            id: list_id,
                            span: *group_span,
                            origin: NameOrigin::Foreign { kind: NameKind::Struct },
                            // NOTE: It has to be session.lang_items.get("type.List"), but we don't have the lang item yet.
                            //       So we first use Prelude, then inter-hir will replace it with the lang item.
                            def_span: Span::Prelude(list_id),
                        },
                        fields: vec![],
                    },
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
            ast::Type::Func { fn_constructor, group_span, params: ast_params, r#return: ast_return } => {
                let mut has_error = false;
                let mut params = Vec::with_capacity(ast_params.len());
                let fn_constructor = match Path::from_ast(fn_constructor, session) {
                    Ok(path) => Some(path),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };

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
                        fn_constructor: fn_constructor.unwrap(),
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
            Type::Path(p) => p.error_span_narrow(),
            Type::Param { constructor, group_span, .. } => {
                constructor.error_span_wide().merge(*group_span)
            },
            Type::Tuple { group_span, .. } => *group_span,
            Type::Func { fn_constructor, .. } => fn_constructor.error_span_narrow(),
            Type::Wildcard(span) | Type::Never(span) => *span,
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Type::Path(p) => p.error_span_wide(),
            Type::Param { constructor, group_span, .. } => {
                constructor.error_span_wide().merge(*group_span)
            },
            Type::Tuple { group_span, .. } => *group_span,
            Type::Func { fn_constructor, group_span, r#return, .. } => {
                fn_constructor.error_span_wide()
                    .merge(*group_span)
                    .merge(r#return.error_span_wide())
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
            Type::Path(p) => {
                p.replace_name_and_span(name, span);
            },
            _ => todo!(),
        }
    }
}
