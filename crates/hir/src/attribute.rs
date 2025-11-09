use crate::{Expr, Session};
use sodigy_error::{Error, ErrorKind};
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
    pub public: Public,
}

impl Attribute {
    pub fn from_ast(
        ast_attribute: &ast::Attribute,
        session: &mut Session,
        rule: AttributeRule,

        // span of `fn`, `let`, `enum`, `struct`, ...
        item_keyword_span: Span,
    ) -> Result<Attribute, ()> {
        let mut has_error = false;
        let doc_comment = match (rule.doc_comment, ast_attribute.doc_comment) {
            (Requirement::Must, None) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::MissingDocComment,
                    spans: item_keyword_span.simple_error(),
                    note: None,
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
                    note: None,
                });
                None
            },
            _ => ast_attribute.doc_comment.clone(),
        };
        let public = todo!();
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
                        session.errors.push();
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
                                        Some((requirement, arg_type)) => {
                                            if let Requirement::Never = requirement {
                                                has_error = true;
                                                session.errors.push();
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
                                                Ok(arg) => match check_arg_type(&arg, *arg_type, session) {
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

                            for (keyword, (requirement, _)) in rule.keyword_args.iter() {
                                if let Requirement::Must = requirement {
                                    if spans_by_keyword.get(keyword).is_none() {
                                        session.errors.push(Error {
                                            kind: ErrorKind::MissingKeywordArgument(*keyword),
                                            spans: merged_span.simple_error(),
                                            note: None,
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
                                            Ok(arg) => match check_arg_type(&arg, arg_type, session) {
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

                                    decorators.insert(name, Decorator {
                                        args,
                                        keyword_args,
                                    });
                                },
                                Err((error_kind, error_span)) => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: error_kind,
                                        spans: error_span,
                                        note: None,
                                    });
                                },
                            }
                        },
                        (_, None) => {
                            decorators.insert(name, Decorator {
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
                        spans: _,
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
                errors.push(Error {
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
                public,
            })
        }
    }
}

pub struct AttributeRule {
    pub doc_comment: Requirement,
    pub publicity: Requirement,
    pub decorators: HashMap<Vec<InternedString>, DecoratorRule>,
}

#[derive(Clone, Copy, Debug)]
pub enum Requirement {
    Must,
    Maybe,
    Never,
}

#[derive(Clone, Debug)]
pub struct Public;

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
    pub arg_type: ArgType,

    pub keyword_args: HashMap<InternedString, (Requirement, ArgType)>,
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
    todo!()
}

fn check_arg_type(arg: &Expr, arg_type: ArgType, session: &mut Session) -> Result<(), ()> {
    todo!()
}
