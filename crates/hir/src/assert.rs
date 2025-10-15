use crate::{Expr, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Assert {
    pub name: Option<InternedString>,
    pub note: Option<InternedString>,
    pub keyword_span: Span,
    pub value: Expr,

    // By default, assertions are enabled only in debug profile.
    // If it has `@always` decorator, it's always enabled.
    pub always: bool,
}

#[derive(Clone, Debug)]
pub struct AssertAttribute {
    pub name: Option<InternedString>,
    pub note: Option<InternedString>,
    pub always: bool,
}

impl Default for AssertAttribute {
    fn default() -> Self {
        AssertAttribute {
            name: None,
            note: None,
            always: false,
        }
    }
}

impl Assert {
    pub fn from_ast(ast_assert: &ast::Assert, session: &mut Session) -> Result<Assert, ()> {
        let mut has_error = false;

        let attribute = match AssertAttribute::from_ast(&ast_assert.attribute, session, ast_assert.keyword_span) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                AssertAttribute::default()
            },
        };

        let is_in_debug_context_prev = session.is_in_debug_context;
        session.is_in_debug_context = !attribute.always;

        let value = match Expr::from_ast(&ast_assert.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        session.is_in_debug_context = is_in_debug_context_prev;

        if has_error {
            Err(())
        }

        else {
            Ok(Assert {
                name: attribute.name,
                note: attribute.note,
                keyword_span: ast_assert.keyword_span,
                value: value.unwrap(),
                always: attribute.always,
            })
        }
    }
}

impl AssertAttribute {
    pub fn from_ast(
        ast_attribute: &ast::Attribute,
        session: &mut Session,
        assert_span: Span,
    ) -> Result<AssertAttribute, ()> {
        let mut name = None;
        let mut note = None;
        let mut always = false;
        let mut has_error = false;

        // Used for error messages.
        let mut name_span_map = HashMap::new();

        for decorator in ast_attribute.decorators.iter() {
            let (d_name, name_span) = decorator.name[0];

            match d_name.try_unintern_short_string() {
                Some(d) if d == b"always" => {
                    if always {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::RedundantDecorator(d_name),
                            span: name_span,
                            extra_span: Some(*name_span_map.get(&d_name).unwrap()),
                            ..Error::default()
                        });
                    }

                    always = true;
                    name_span_map.insert(d_name, name_span);
                },
                Some(d) if d == b"name" => {
                    if name.is_some() {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::RedundantDecorator(d_name),
                            span: name_span,
                            extra_span: Some(*name_span_map.get(&d_name).unwrap()),
                            ..Error::default()
                        });
                    }

                    name_span_map.insert(d_name, name_span);
                    todo!();
                },
                Some(d) if d == b"note" => {
                    if note.is_some() {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::RedundantDecorator(d_name),
                            span: name_span,
                            extra_span: Some(*name_span_map.get(&d_name).unwrap()),
                            ..Error::default()
                        });
                    }

                    name_span_map.insert(d_name, name_span);
                    todo!();
                },
                _ => {
                    has_error = true;
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidDecorator(d_name),
                        span: name_span,
                        ..Error::default()
                    });
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(AssertAttribute {
                name,
                note,
                always,
            })
        }
    }
}
