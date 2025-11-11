use crate::{CallArg, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use sodigy_token::{
    Delim,
    Keyword,
    Punct,
    Token,
    TokenKind,
};

#[derive(Clone, Debug)]
pub struct Attribute {
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
    pub visibility: Option<Visibility>,
}

impl Attribute {
    pub fn new() -> Self {
        Attribute {
            doc_comment: None,
            decorators: vec![],
            visibility: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.doc_comment.is_none() && self.decorators.is_empty() && self.visibility.is_none()
    }
}

#[derive(Clone, Debug)]
pub struct DocComment(pub Vec<DocCommentLine>);

impl DocComment {
    pub fn new(lines: Vec<DocCommentLine>) -> Self {
        DocComment(lines)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DocCommentLine {
    pub content: InternedString,
    pub content_span: Span,
    pub marker_span: Span,
}

impl DocCommentLine {
    pub fn new(content: InternedString, entire_span: Span) -> Self {
        let (marker_span, content_span) = match entire_span {
            Span::Range { file, start, end } if start + 3 <= end => (
                Span::Range { file, start, end: start + 3 },
                Span::Range { file, start: start + 3, end },
            ),
            _ => unreachable!(),
        };

        DocCommentLine {
            content,
            content_span,
            marker_span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decorator {
    // Rust attributes support paths, like `#[rustfmt::skip]`, but Sodigy doesn't.
    // That's because rust supports user-defined attributes, but Sodigy doesn't.
    pub name: InternedString,
    pub name_span: Span,

    // `#[public]` and `#[public()]` are different!
    pub args: Option<Vec<CallArg>>,
    pub arg_group_span: Option<Span>,
}

#[derive(Clone, Debug)]
pub struct Visibility {
    pub keyword_span: Span,

    // `pub` and `pub()` are different!
    pub args: Option<Vec<(InternedString, Span)>>,
    pub arg_group_span: Option<Span>,
}

impl<'t> Tokens<'t> {
    pub fn collect_attribute(&mut self, top_level: bool) -> Result<Attribute, Vec<Error>> {
        let mut errors = vec![];
        let mut doc_comments = vec![];
        let mut decorators = vec![];
        let mut visibility = None;
        let mut module_doc_error = false;

        loop {
            match self.peek() {
                Some(Token { kind: TokenKind::DocComment { doc, top_level: top_level_ }, span }) => {
                    if !top_level && *top_level_ {
                        // If the programmer accidentally wrote a very long module document at wrong place,
                        // it would generate a very long error message. I want to prevent that.
                        if !module_doc_error {
                            errors.push(Error {
                                kind: ErrorKind::ModuleDocCommentNotAtTop,
                                spans: span.simple_error(),
                                note: None,
                            });
                            module_doc_error = true;
                        }
                    }

                    doc_comments.push(DocCommentLine::new(*doc, *span));
                    self.cursor += 1;
                },
                Some(Token { kind: TokenKind::Group {
                    delim: delim @ (Delim::Decorator | Delim::ModuleDecorator),
                    tokens
                }, span }) => {
                    let top_level_ = matches!(delim, Delim::ModuleDecorator);

                    if !top_level && top_level_ {
                        errors.push(Error {
                            kind: ErrorKind::ModuleDecoratorNotAtTop,
                            spans: span.simple_error(),
                            note: None,
                        });
                    }

                    let group_span = *span;
                    let mut tokens = Tokens::new(tokens, group_span.end());

                    match tokens.parse_decorator() {
                        Ok(decorator) => {
                            decorators.push(decorator);
                        },
                        Err(e) => {
                            errors.extend(e);
                        },
                    }

                    self.cursor += 1;

                    if let Some(Token { kind: TokenKind::Punct(Punct::Semicolon), span }) = self.peek() {
                        errors.push(Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Nothing,
                                got: ErrorToken::Punct(Punct::Semicolon),
                            },
                            spans: vec![RenderableSpan {
                                span: *span,
                                auxiliary: false,
                                note: Some(String::from("Remove this `;`.")),
                            }],
                            note: Some(String::from("Don't put a semicolon after a decorator.")),
                        });
                        self.cursor += 1;
                        return Err(errors);
                    }
                },
                Some(Token { kind: TokenKind::Keyword(Keyword::Pub), span }) => {
                    if !top_level {
                        let keyword_span = *span;
                        self.cursor += 1;

                        match self.peek() {
                            Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }) => todo!(),
                            _ => {},
                        }

                        visibility = Some(Visibility {
                            keyword_span,
                            args: None,
                            arg_group_span: None,
                        });
                    }

                    break;
                },
                _ => {
                    break;
                },
            }
        }

        if errors.is_empty() {
            let doc_comment = match doc_comments.len() {
                0 => None,
                _ => Some(DocComment::new(doc_comments)),
            };

            Ok(Attribute {
                doc_comment,
                decorators,
                visibility,
            })
        }

        else {
            Err(errors)
        }
    }

    pub fn parse_decorator(&mut self) -> Result<Decorator, Vec<Error>> {
        let (name, name_span) = self.pop_name_and_span()?;

        match self.peek() {
            Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }) => {
                let group_span = *span;
                let mut tokens = Tokens::new(tokens, group_span.end());
                let args = tokens.parse_call_args()?;
                let result = Decorator {
                    name,
                    name_span,
                    args: Some(args),
                    arg_group_span: Some(group_span),
                };
                self.cursor += 1;

                match self.peek() {
                    Some(t) => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Nothing,
                                got: (&t.kind).into(),
                            },
                            spans: t.span.simple_error(),
                            note: None,
                        }]);
                    },
                    None => {
                        return Ok(result);
                    },
                }
            },
            Some(t) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Group(Delim::Parenthesis),
                        got: (&t.kind).into(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            None => {
                return Ok(Decorator {
                    name,
                    name_span,
                    args: None,
                    arg_group_span: None,
                });
            },
        }
    }
}
