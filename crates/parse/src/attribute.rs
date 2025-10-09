use crate::{CallArg, Tokens};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Attribute {
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

impl Attribute {
    pub fn new() -> Self {
        Attribute {
            doc_comment: None,
            decorators: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.doc_comment.is_none() && self.decorators.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct DocComment {
    pub content: InternedString,
    pub span: Span,
}

impl DocComment {
    pub fn new(content: InternedString, span: Span) -> Self {
        DocComment { content, span }
    }
}

#[derive(Clone, Debug)]
pub struct Decorator {
    pub name: Vec<(InternedString, Span)>,  // dotted name, like `test.eq`
    pub name_span: Span,  // merged span of names

    // `@public` and `@public()` are different!
    pub args: Option<Vec<CallArg>>,
}

impl Decorator {
    pub fn new_with_args(name: Vec<(InternedString, Span)>, name_span: Span, args: Vec<CallArg>) -> Self {
        Decorator {
            name,
            name_span,
            args: Some(args),
        }
    }

    pub fn new_without_args(name: Vec<(InternedString, Span)>, name_span: Span) -> Self {
        Decorator {
            name,
            name_span,
            args: None,
        }
    }
}

impl<'t> Tokens<'t> {
    // If there are multiple doc comments, it throws an error.
    pub fn collect_attribute(&mut self) -> Result<Attribute, Vec<Error>> {
        let mut errors = vec![];
        let mut doc_comment_buffer = vec![];
        let mut decorator_buffer = vec![];

        loop {
            match self.peek2() {
                (Some(Token { kind: TokenKind::DocComment(doc), span }), _) => {
                    doc_comment_buffer.push(DocComment::new(*doc, *span));
                    self.cursor += 1;
                },
                (
                    Some(Token { kind: TokenKind::Punct(Punct::At), span: span1 }),
                    Some(Token { kind: TokenKind::Identifier(dec), span: span2 }),
                ) => {
                    let span = span1.merge(*span2);
                    let mut name = vec![(*dec, span)];
                    let mut name_span = span;
                    self.cursor += 2;

                    loop {
                        match self.peek2() {
                            (
                                Some(Token { kind: TokenKind::Punct(Punct::Dot), .. }),
                                Some(Token { kind: TokenKind::Identifier(dec), span }),
                            ) => {
                                name.push((*dec, *span));
                                name_span = name_span.merge(*span);
                                self.cursor += 2;
                            },
                            (Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }), _) => {
                                let mut tokens = Tokens::new(tokens, span.end());

                                match tokens.parse_call_args() {
                                    Ok(args) => {
                                        decorator_buffer.push(Decorator::new_with_args(name, name_span, args));
                                    },
                                    Err(e) => {
                                        errors.extend(e);
                                    },
                                }

                                self.cursor += 1;
                                break;
                            },
                            _ => {
                                decorator_buffer.push(Decorator::new_without_args(name, name_span));
                                break;
                            },
                        }
                    }
                },
                _ => {
                    break;
                },
            }
        }

        if errors.is_empty() {
            let doc_comment = match doc_comment_buffer.len() {
                0 => None,
                1 => Some(doc_comment_buffer[0].clone()),
                _ => {
                    errors.push(Error {
                        kind: ErrorKind::DocCommentForNothing,
                        span: doc_comment_buffer[0].span,
                        ..Error::default()
                    });
                    return Err(errors);
                },
            };

            Ok(Attribute {
                doc_comment,
                decorators: decorator_buffer,
            })
        }

        else {
            Err(errors)
        }
    }
}
