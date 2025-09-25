use crate::{CallArg, Tokens};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Token, TokenKind};

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
    pub name: InternedString,
    pub name_span: Span,

    // `@public` and `@public()` are different!
    pub args: Option<Vec<CallArg>>,
}

impl Decorator {
    pub fn new_with_args(name: InternedString, name_span: Span, args: Vec<CallArg>) -> Self {
        Decorator {
            name,
            name_span,
            args: Some(args),
        }
    }

    pub fn new_without_args(name: InternedString, name_span: Span) -> Self {
        Decorator {
            name,
            name_span,
            args: None,
        }
    }
}

impl<'t> Tokens<'t> {
    // If there are multiple doc comments, it throws an error.
    pub fn collect_doc_comment_and_decorators(&mut self) -> Result<(Option<DocComment>, Vec<Decorator>), Vec<Error>> {
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
                    Some(Token { kind: TokenKind::Decorator(dec), span: span1 }),
                    Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span: span2 }),
                ) => {
                    let mut tokens = Tokens::new(tokens, span2.end());

                    match tokens.parse_call_args() {
                        Ok(args) => {
                            decorator_buffer.push(Decorator::new_with_args(*dec, *span1, args));
                        },
                        Err(e) => {
                            errors.extend(e);
                        },
                    }

                    self.cursor += 2;
                },
                (
                    Some(Token { kind: TokenKind::Decorator(dec), span }),
                    _,
                ) => {
                    decorator_buffer.push(Decorator::new_without_args(*dec, *span));
                    self.cursor += 1;
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

            Ok((doc_comment, decorator_buffer))
        }

        else {
            Err(errors)
        }
    }
}
