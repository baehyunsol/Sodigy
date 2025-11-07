use crate::{Expr, Session};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_parse::{self as ast, CallArg};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Clone, Debug)]
pub struct Assert {
    // A name of an assertion must be a string literal, but you can use
    // any string expression as a note.
    // e.g. `@name("test1")` is valid,
    //      `@name(f"test{i}")` is not valid,
    //      `@name(test1)` is not valid,
    //      `@note("It is a test")` is valid,
    //      `@note(f"check {a}+{b}={a+b}")` is valid,
    //      `@note(3 + 4)` is not valid (type error).
    // I chose this way because
    //
    // 1. In order to create a test harness, it has to be easy for the compiler
    //    to know the name of the assertions. So, I don't want a runtime-evaluation.
    // 2. If it uses an identifier instead of a string literal, there are much less
    //    characters to use. For example, the user might want to use colons in the
    //    name of an assertion.
    // 3. `@note` must be very flexible.
    pub name: Option<InternedString>,
    pub note: Option<Expr>,

    pub keyword_span: Span,
    pub value: Expr,

    // By default, assertions are enabled only in debug profile.
    // If it has `@always` decorator, it's always enabled.
    pub always: bool,
}

#[derive(Clone, Debug)]
pub struct AssertAttribute {
    pub name: Option<InternedString>,
    pub note: Option<Expr>,
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
        let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        for decorator in ast_attribute.decorators.iter() {
            let (d_name, name_span) = decorator.name[0];

            match spans_by_name.entry(d_name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![name_span]);
                },
            }

            match d_name.try_unintern_short_string() {
                Some(d) if d == b"always" => {
                    if always {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::RedundantDecorator(d_name),
                            spans: spans_by_name.get(&d_name).unwrap_or(&vec![]).iter().map(
                                |span| RenderableSpan {
                                    span: *span,
                                    auxiliary: false,
                                    note: None,
                                }
                            ).collect(),
                            note: None,
                        });
                    }

                    session.error_if_decorator_name_too_long(&mut has_error, &decorator.name, 1);
                    always = true;
                },
                Some(d) if d == b"name" || d == b"note" => {
                    if d == b"name" && name.is_some() || d == b"note" && note.is_some() {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::RedundantDecorator(d_name),
                            spans: spans_by_name.get(&d_name).unwrap_or(&vec![]).iter().map(
                                |span| RenderableSpan {
                                    span: *span,
                                    auxiliary: false,
                                    note: None,
                                }
                            ).collect(),
                            note: None,
                        });
                    }

                    session.error_if_decorator_name_too_long(&mut has_error, &decorator.name, 1);

                    match &decorator.args {
                        Some(args) => {
                            match args.get(0) {
                                Some(CallArg { keyword: Some((k, span)), .. }) => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::InvalidKeywordArgument(*k),
                                        spans: span.simple_error(),
                                        note: None,
                                    });
                                },
                                Some(CallArg { arg: arg @ ast::Expr::String { s, binary: false, .. }, .. }) => {
                                    if d == b"name" {
                                        name = Some(*s);
                                    }

                                    else {
                                        match Expr::from_ast(arg, session) {
                                            Ok(arg) => {
                                                note = Some(arg);
                                            },
                                            Err(()) => {
                                                has_error = true;
                                            },
                                        }
                                    }
                                },
                                Some(CallArg { arg, .. }) => {
                                    if d == b"name" {
                                        has_error = true;
                                        session.errors.push(Error {
                                            kind: ErrorKind::UnexpectedToken {
                                                expected: ErrorToken::String,
                                                got: ErrorToken::Expr,
                                            },
                                            spans: arg.error_span().simple_error(),
                                            note: Some(String::from("A name of an assertion must be a compile-time-evaluable string.")),
                                        });
                                    }

                                    else {
                                        match Expr::from_ast(arg, session) {
                                            Ok(arg) => {
                                                note = Some(arg);
                                            },
                                            Err(()) => {
                                                has_error = true;
                                            },
                                        }
                                    }
                                },
                                None => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::MissingArgument {
                                            expected: 1,
                                            got: 0,
                                        },
                                        spans: decorator.arg_group_span.unwrap().end().simple_error(),
                                        note: None,
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
                                    spans: args[1].arg.error_span().simple_error(),
                                    note: None,
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
                                spans: name_span.simple_error(),
                                note: None,
                            });
                        },
                    }
                },
                _ => {
                    has_error = true;
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidDecorator(d_name),
                        spans: name_span.simple_error(),
                        note: None,
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
