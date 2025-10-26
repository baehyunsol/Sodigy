use crate::{CallArg, Tokens};
use sodigy_error::Error;
use sodigy_span::Span;
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
    pub public: Option<Public>,
}

impl Attribute {
    pub fn new() -> Self {
        Attribute {
            doc_comment: None,
            decorators: vec![],
            public: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.doc_comment.is_none() && self.decorators.is_empty() && self.public.is_none()
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
    pub name: Vec<(InternedString, Span)>,  // dotted name, like `test.eq`
    pub name_span: Span,  // merged span of names

    // `@public` and `@public()` are different!
    pub args: Option<Vec<CallArg>>,
    pub arg_group_span: Option<Span>,
}

impl Decorator {
    pub fn new_with_args(
        name: Vec<(InternedString, Span)>,
        name_span: Span,
        args: Vec<CallArg>,
        arg_group_span: Span,
    ) -> Self {
        Decorator {
            name,
            name_span,
            args: Some(args),
            arg_group_span: Some(arg_group_span),
        }
    }

    pub fn new_without_args(name: Vec<(InternedString, Span)>, name_span: Span) -> Self {
        Decorator {
            name,
            name_span,
            args: None,
            arg_group_span: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Public {
    pub keyword_span: Span,

    // `pub` and `pub()` are different!
    pub args: Option<Vec<(InternedString, Span)>>,
    pub arg_group_span: Option<Span>,
}

impl<'t> Tokens<'t> {
    // If there are multiple doc comments, it throws an error.
    pub fn collect_attribute(&mut self) -> Result<Attribute, Vec<Error>> {
        let mut errors = vec![];
        let mut doc_comment_buffer = vec![];
        let mut decorator_buffer = vec![];
        let mut public = None;

        loop {
            match self.peek2() {
                (Some(Token { kind: TokenKind::DocComment(doc), span }), _) => {
                    doc_comment_buffer.push(DocCommentLine::new(*doc, *span));
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
                                let group_span = *span;
                                let mut tokens = Tokens::new(tokens, span.end());

                                match tokens.parse_call_args() {
                                    Ok(args) => {
                                        decorator_buffer.push(Decorator::new_with_args(name, name_span, args, group_span));
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
                (Some(Token { kind: TokenKind::Keyword(Keyword::Pub), span }), _) => {
                    let keyword_span = *span;
                    self.cursor += 1;

                    match self.peek() {
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }) => todo!(),
                        _ => {},
                    }

                    public = Some(Public {
                        keyword_span,
                        args: None,
                        arg_group_span: None,
                    });
                    break;
                },
                _ => {
                    break;
                },
            }
        }

        if errors.is_empty() {
            let doc_comment = match doc_comment_buffer.len() {
                0 => None,
                _ => Some(DocComment::new(doc_comment_buffer)),
            };

            Ok(Attribute {
                doc_comment,
                decorators: decorator_buffer,
                public,
            })
        }

        else {
            Err(errors)
        }
    }
}
