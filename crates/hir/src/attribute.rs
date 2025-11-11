use crate::{Expr, Session};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_parse::{self as ast, DocComment};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{
    InternedString,
    intern_string,
    unintern_string,
};
use std::collections::hash_map::{Entry, HashMap};

// `ast::Attribute` is first lowered to this type. It does some basic
// checks (redundant names, undefined names, arguments).
// Each item extracts extra information from this type.
pub struct Attribute {
    pub doc_comment: Option<DocComment>,
    pub decorators: HashMap<Vec<InternedString>, Decorator>,
    pub visibility: Visibility,
}

impl Attribute {
    pub fn new() -> Attribute {
        Attribute {
            doc_comment: None,
            decorators: HashMap::new(),
            visibility: Visibility::private(),
        }
    }

    pub fn from_ast(
        ast_attribute: &ast::Attribute,
        session: &mut Session,
        rule: &AttributeRule,

        // span of `fn`, `let`, `enum`, `struct`, ...
        item_keyword_span: Span,
    ) -> Result<Attribute, ()> {
        let mut has_error = false;
        let doc_comment = match (rule.doc_comment, &ast_attribute.doc_comment) {
            (Requirement::Must, None) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::MissingDocComment,
                    spans: item_keyword_span.simple_error(),
                    note: rule.doc_comment_error_note.clone(),
                });
                None
            },
            (Requirement::Never, Some(doc_comment)) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::DocCommentNotAllowed,
                    spans: vec![
                        RenderableSpan {
                            span: item_keyword_span,
                            auxiliary: true,
                            note: Some(String::from("You can't add doc comment to this.")),
                        },
                        RenderableSpan {
                            span: doc_comment.0[0].marker_span,
                            auxiliary: false,
                            note: None,
                        },
                    ],
                    note: rule.doc_comment_error_note.clone(),
                });
                None
            },
            _ => ast_attribute.doc_comment.clone(),
        };
        let visibility = match (rule.visibility, &ast_attribute.visibility) {
            (Requirement::Must, None) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::MissingVisibility,
                    spans: item_keyword_span.simple_error(),
                    note: rule.visibility_error_note.clone(),
                });
                Visibility::private()
            },
            (Requirement::Never, Some(ast_visibility)) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::CannotBePublic,
                    spans: vec![
                        RenderableSpan {
                            span: item_keyword_span,
                            auxiliary: true,
                            note: Some(String::from("This cannot be public.")),
                        },
                        RenderableSpan {
                            span: ast_visibility.keyword_span,
                            auxiliary: false,
                            note: None,
                        },
                    ],
                    note: rule.visibility_error_note.clone(),
                });

                match Visibility::from_ast(&ast_visibility, session) {
                    Ok(visibility) => visibility,
                    Err(()) => {
                        has_error = true;
                        Visibility::private()
                    },
                }
            },
            (_, None) => Visibility::private(),
            (_, Some(ast_visibility)) => match Visibility::from_ast(&ast_visibility, session) {
                Ok(visibility) => visibility,
                Err(()) => {
                    has_error = true;
                    Visibility::private()
                },
            },
        };

        let mut decorators = HashMap::with_capacity(ast_attribute.decorators.len());

        // for error messages
        let mut spans_by_name: HashMap<Vec<InternedString>, Vec<Span>> = HashMap::new();

        for ast_decorator in ast_attribute.decorators.iter() {
            let name: Vec<InternedString> = ast_decorator.name.iter().map(|(name, _)| *name).collect();
            let merged_span = ast_decorator.name.iter().map(
                |(_, span)| *span
            ).fold(
                ast_decorator.name[0].1,
                |folded, span| folded.merge(span),
            );

            match rule.decorators.get(&name) {
                Some(rule) => {
                    if let Requirement::Never = rule.requirement {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::UnexpectedDecorator(join_decorator_name(&name, &session)),
                            spans: merged_span.simple_error(),
                            note: None,
                        });
                    }

                    match (rule.arg_requirement, &ast_decorator.args) {
                        (Requirement::Must, None) => {
                            has_error = true;
                            session.errors.push(Error {
                                kind: ErrorKind::MissingArgument {
                                    expected: 1,  // how many?
                                    got: 0,
                                },
                                spans: merged_span.simple_error(),
                                note: None,
                            });
                        },
                        (Requirement::Never, Some(ast_args)) => {
                            has_error = true;
                            session.errors.push(Error {
                                kind: ErrorKind::UnexpectedArgument {
                                    expected: 0,
                                    got: ast_args.len(),
                                },
                                spans: vec![
                                    RenderableSpan {
                                        span: merged_span,
                                        auxiliary: true,
                                        note: Some(String::from("It requires no arguments.")),
                                    },
                                    RenderableSpan {
                                        span: ast_decorator.arg_group_span.unwrap(),
                                        auxiliary: false,
                                        note: Some(String::from("Remove this parenthesis.")),
                                    },
                                ],
                                note: None,
                            });
                        },
                        (_, Some(ast_args)) => {
                            let mut keyword_args: HashMap<InternedString, Expr> = HashMap::new();
                            let mut positional_args: Vec<&ast::Expr> = vec![];
                            let mut spans_by_keyword: HashMap<InternedString, Vec<Span>> = HashMap::new();

                            for ast_arg in ast_args.iter() {
                                match ast_arg.keyword {
                                    Some((keyword, span)) => match rule.keyword_args.get(&keyword) {
                                        Some(KeywordArgRule {
                                            requirement,
                                            requirement_error_note,
                                            arg_type,
                                            arg_type_error_note,
                                        }) => {
                                            if let Requirement::Never = requirement {
                                                has_error = true;
                                                session.errors.push(Error {
                                                    kind: ErrorKind::InvalidKeywordArgument(keyword),
                                                    spans: span.simple_error(),
                                                    note: requirement_error_note.clone(),
                                                });
                                            }

                                            match spans_by_keyword.entry(keyword) {
                                                Entry::Occupied(mut e) => {
                                                    e.get_mut().push(span);
                                                },
                                                Entry::Vacant(e) => {
                                                    e.insert(vec![span]);
                                                },
                                            }

                                            match Expr::from_ast(&ast_arg.arg, session) {
                                                Ok(arg) => match check_arg_type(&arg, *arg_type, arg_type_error_note, session) {
                                                    Ok(()) => {
                                                        keyword_args.insert(keyword, arg);
                                                    },
                                                    Err(()) => {
                                                        has_error = true;
                                                    },
                                                },
                                                Err(()) => {
                                                    has_error = true;
                                                },
                                            }
                                        },
                                        None => {
                                            has_error = true;
                                            session.errors.push(Error {
                                                kind: ErrorKind::InvalidKeywordArgument(keyword),
                                                spans: span.simple_error(),
                                                note: None,
                                            });
                                        },
                                    },
                                    None => {
                                        positional_args.push(&ast_arg.arg);
                                    },
                                }
                            }

                            for (keyword, spans) in spans_by_keyword.iter() {
                                if spans.len() > 1 {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::KeywordArgumentRepeated(*keyword),
                                        spans: spans.iter().map(
                                            |span| RenderableSpan {
                                                span: *span,
                                                auxiliary: false,
                                                note: None,
                                            }
                                        ).collect(),
                                        note: None,
                                    });
                                }
                            }

                            for (keyword, KeywordArgRule { requirement, requirement_error_note, .. }) in rule.keyword_args.iter() {
                                if let Requirement::Must = requirement {
                                    if spans_by_keyword.get(keyword).is_none() {
                                        session.errors.push(Error {
                                            kind: ErrorKind::MissingKeywordArgument(*keyword),
                                            spans: merged_span.simple_error(),
                                            note: requirement_error_note.clone(),
                                        });
                                    }
                                }
                            }

                            let count_rule = match (rule.arg_count, positional_args.len()) {
                                (ArgCount::Zero, 1..) => Err((
                                    ErrorKind::UnexpectedArgument {
                                        expected: 0,
                                        got: positional_args.len(),
                                    },
                                    positional_args.iter().map(
                                        |arg| RenderableSpan {
                                            span: arg.error_span(),
                                            auxiliary: false,
                                            note: None,
                                        }
                                    ).collect(),
                                )),
                                (ArgCount::Eq(n), m) if n > m => Err((
                                    ErrorKind::MissingArgument {
                                        expected: n,
                                        got: m,
                                    },
                                    merged_span.simple_error(),
                                )),
                                (ArgCount::Eq(n), m) if n < m => Err((
                                    ErrorKind::UnexpectedArgument {
                                        expected: n,
                                        got: m,
                                    },
                                    positional_args[n..].iter().map(
                                        |arg| RenderableSpan {
                                            span: arg.error_span(),
                                            auxiliary: false,
                                            note: None,
                                        }
                                    ).collect(),
                                )),
                                (ArgCount::Gt(n), m) if n >= m => Err((
                                    ErrorKind::MissingArgument {
                                        expected: n + 1,
                                        got: m,
                                    },
                                    merged_span.simple_error(),
                                )),
                                (ArgCount::Lt(n), m) if n <= m => Err((
                                    ErrorKind::UnexpectedArgument {
                                        expected: n - 1,
                                        got: m,
                                    },
                                    positional_args[(n - 1)..].iter().map(
                                        |arg| RenderableSpan {
                                            span: arg.error_span(),
                                            auxiliary: false,
                                            note: None,
                                        }
                                    ).collect(),
                                )),
                                _ => Ok(()),
                            };

                            match count_rule {
                                Ok(()) => {
                                    let mut args = Vec::with_capacity(positional_args.len());

                                    for ast_arg in positional_args.iter() {
                                        match Expr::from_ast(ast_arg, session) {
                                            Ok(arg) => match check_arg_type(&arg, rule.arg_type, &rule.arg_type_error_note, session) {
                                                Ok(()) => {
                                                    args.push(arg);
                                                },
                                                Err(()) => {
                                                    has_error = true;
                                                },
                                            },
                                            Err(()) => {
                                                has_error = true;
                                            },
                                        }
                                    }

                                    decorators.insert(name.clone(), Decorator {
                                        args,
                                        keyword_args,
                                    });
                                },
                                Err((error_kind, error_span)) => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: error_kind,
                                        spans: error_span,
                                        note: rule.arg_count_error_note.clone(),
                                    });
                                },
                            }
                        },
                        (_, None) => {
                            decorators.insert(name.clone(), Decorator {
                                args: vec![],
                                keyword_args: HashMap::new(),
                            });
                        },
                    }
                },
                None => {
                    // TODO: try `rule.decorators.get(&name[..i])` to generate a better error message
                    has_error = true;
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidDecorator(join_decorator_name(&name, session)),
                        spans: merged_span.simple_error(),
                        note: None,
                    });
                },
            }

            match spans_by_name.entry(name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(merged_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![merged_span]);
                },
            }
        }

        for (name, spans) in spans_by_name.iter() {
            if spans.len() > 1 {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::RedundantDecorator(join_decorator_name(name, session)),
                    spans: spans.iter().map(
                        |span| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect(),
                    note: None,
                });
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Attribute {
                doc_comment,
                decorators,
                visibility,
            })
        }
    }

    pub fn built_in(&self, intermediate_dir: &str) -> bool {
        self.decorators.contains_key(&vec![intern_string(b"built_in", intermediate_dir).unwrap()])
    }

    pub fn no_type(&self, intermediate_dir: &str) -> bool {
        self.decorators.contains_key(&vec![intern_string(b"no_type", intermediate_dir).unwrap()])
    }

    pub fn lang_item(&self, intermediate_dir: &str) -> Option<String> {
        match self.decorators.get(&vec![intern_string(b"lang_item", intermediate_dir).unwrap()]) {
            Some(d) => match d.args.get(0) {
                Some(Expr::String { s, .. }) => Some(String::from_utf8_lossy(&unintern_string(*s, intermediate_dir).unwrap().unwrap()).to_string()),
                _ => unreachable!(),
            },
            None => None,
        }
    }

    pub fn lang_item_generics(&self, intermediate_dir: &str) -> Option<Vec<String>> {
        match self.decorators.get(&vec![intern_string(b"lang_item_generics", intermediate_dir).unwrap()]) {
            Some(d) => Some(d.args.iter().map(
                |arg| match arg {
                    Expr::String { s, .. } => String::from_utf8_lossy(&unintern_string(*s, intermediate_dir).unwrap().unwrap()).to_string(),
                    _ => unreachable!(),
                }
            ).collect()),
            None => None,
        }
    }
}

pub struct AttributeRule {
    pub doc_comment: Requirement,
    pub doc_comment_error_note: Option<String>,
    pub visibility: Requirement,
    pub visibility_error_note: Option<String>,
    pub decorators: HashMap<Vec<InternedString>, DecoratorRule>,
}

impl AttributeRule {
    pub fn add_std_rules(&mut self, intermediate_dir: &str) {
        for (name, mut decorator) in [
            (
                "built_in",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Never,
                    ..DecoratorRule::default()
                },
            ),
            (
                "no_type",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Never,
                    ..DecoratorRule::default()
                },
            ),
            (
                "lang_item",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Eq(1),
                    arg_count_error_note: Some(String::from("An item can have at most 1 lang item.")),
                    arg_type: ArgType::StringLiteral,
                    arg_type_error_note: Some(String::from("A lang item must be a string literal, which is compile-time-evaluable.")),
                    ..DecoratorRule::default()
                },
            ),
            (
                "lang_item_generics",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Gt(0),
                    arg_count_error_note: None,
                    arg_type: ArgType::StringLiteral,
                    arg_type_error_note: Some(String::from("A lang item must be a string literal, which is compile-time-evaluable.")),
                    ..DecoratorRule::default()
                },
            ),
        ] {
            let name = vec![intern_string(name.as_bytes(), intermediate_dir).unwrap()];
            decorator.name = name.clone();
            self.decorators.insert(name, decorator);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Requirement {
    Must,
    Maybe,
    Never,
}

#[derive(Clone, Debug)]
pub struct Visibility {
    pub keyword_span: Option<Span>,
    // TODO: more fields
}

impl Visibility {
    pub fn from_ast(ast_visibility: &ast::Visibility, session: &mut Session) -> Result<Visibility, ()> {
        Ok(Visibility {
            keyword_span: Some(ast_visibility.keyword_span),
            // TODO: more fields
        })
    }

    pub fn private() -> Visibility {
        Visibility {
            keyword_span: None,
        }
    }

    pub fn is_public(&self) -> bool {
        // TODO: more fine-grained visibility control
        self.keyword_span.is_some()
    }
}

pub struct Decorator {
    pub args: Vec<Expr>,
    pub keyword_args: HashMap<InternedString, Expr>,
}

pub struct DecoratorRule {
    pub name: Vec<InternedString>,
    pub requirement: Requirement,

    // `ArgCount::Zero` and `Requirement::Never` are different.
    // `ArgCount::Zero` is `@note()`, while `Requirement::Never` is `@note`.
    pub arg_requirement: Requirement,
    pub arg_count: ArgCount,
    pub arg_count_error_note: Option<String>,
    pub arg_type: ArgType,
    pub arg_type_error_note: Option<String>,

    pub keyword_args: HashMap<InternedString, KeywordArgRule>,
}

impl Default for DecoratorRule {
    fn default() -> DecoratorRule {
        DecoratorRule {
            name: vec![],
            requirement: Requirement::Never,
            arg_requirement: Requirement::Never,
            arg_count: ArgCount::Zero,
            arg_count_error_note: None,
            arg_type: ArgType::Expr,
            arg_type_error_note: None,
            keyword_args: HashMap::new(),
        }
    }
}

pub struct KeywordArgRule {
    pub requirement: Requirement,
    pub requirement_error_note: Option<String>,
    pub arg_type: ArgType,
    pub arg_type_error_note: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum ArgType {
    StringLiteral,
    Expr,
}

#[derive(Clone, Copy, Debug)]
pub enum ArgCount {
    Zero,
    Eq(usize),
    Gt(usize),
    Lt(usize),
}

fn join_decorator_name(name: &[InternedString], session: &Session) -> InternedString {
    let uninterned_name = name.iter().map(
        |name| unintern_string(*name, &session.intermediate_dir).unwrap().unwrap()
    ).collect::<Vec<_>>();
    let joined_name = uninterned_name.join(&(b"."[..]));
    intern_string(&joined_name, &session.intermediate_dir).unwrap()
}

fn check_arg_type(arg: &Expr, arg_type: ArgType, error_note: &Option<String>, session: &mut Session) -> Result<(), ()> {
    match (arg_type, arg) {
        (ArgType::Expr, _) => Ok(()),
        (ArgType::StringLiteral, Expr::String { .. }) => Ok(()),
        (ArgType::StringLiteral, _) => {
            session.errors.push(Error {
                // It's not a type error. An f-string token has type `String`, but it's still an error.
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::String,
                    got: ErrorToken::Expr,
                },
                spans: arg.error_span().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
    }
}
