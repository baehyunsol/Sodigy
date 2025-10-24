use crate::{Expr, Session};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_parse::{self as ast, CallArg};
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

        let attribute = match AssertAttribute::from_ast(&ast_assert.attribute, session) {
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
                Some(d) if d == b"name" || d == b"note" => {
                    if d == b"name" && name.is_some() || d == b"note" && note.is_some() {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::RedundantDecorator(d_name),
                            span: name_span,
                            extra_span: Some(*name_span_map.get(&d_name).unwrap()),
                            ..Error::default()
                        });
                    }

                    name_span_map.insert(d_name, name_span);
                    let mut d_arg = None;

                    match &decorator.args {
                        Some(args) => {
                            match args.get(0) {
                                Some(CallArg { keyword: Some((k, span)), .. }) => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::InvalidKeywordArgument(*k),
                                        span: *span,
                                        ..Error::default()
                                    });
                                },
                                Some(CallArg { arg: ast::Expr::String { s, binary: false, .. }, .. }) => {
                                    d_arg = Some(*s);
                                },
                                Some(CallArg { arg, .. }) => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::UnexpectedToken {
                                            expected: ErrorToken::String,
                                            got: ErrorToken::Expr,
                                        },
                                        span: arg.error_span(),
                                        ..Error::default()
                                    });
                                },
                                None => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::MissingArgument {
                                            expected: 1,
                                            got: 0,
                                        },
                                        span: decorator.arg_group_span.unwrap().end(),
                                        ..Error::default()
                                    });
                                },
                            }

                            if args.len() > 1 {
                                has_error = true;
                                session.errors.push(Error {
                                    kind: ErrorKind::UnexpectedArgument {
                                        expected: 1,
                                        got: args.len(),
                                    },
                                    span: args[1].arg.error_span(),
                                    ..Error::default()
                                });
                            }
                        },
                        None => {
                            has_error = true;
                            session.errors.push(Error {
                                kind: ErrorKind::MissingArgument {
                                    expected: 1,
                                    got: 0,
                                },
                                span: name_span,
                                ..Error::default()
                            });
                        },
                    }

                    if d == b"name" {
                        name = d_arg;
                    }

                    else {
                        note = d_arg;
                    }
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
