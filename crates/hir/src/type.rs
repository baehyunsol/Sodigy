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
            ast::Type::List { r#type, group_span } => {
                // TODO: I want it to be const-evaled
                let list_id = intern_string(b"List");

                Ok(Type::Identifier(IdentWithOrigin {
                    id: list_id,
                    span: *group_span,
                    origin: NameOrigin::Foreign { kind: NameKind::Struct },
                    def_span: Span::Prelude(list_id),
                }))
            },
            ast::Type::Tuple { types: ast_types, group_span } => {
                let mut types = Vec::with_capacity(ast_types.len());
                let mut has_error = false;

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
            _ => panic!("TODO: {ast_type:?}"),
        }
    }
}
