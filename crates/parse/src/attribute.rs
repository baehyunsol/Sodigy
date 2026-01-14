use crate::{Expr, Tokens, Type};
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
    pub args: Option<Vec<DecoratorArg>>,
    pub arg_group_span: Option<Span>,
}

// A decorator argument can be an expression or a type annotation,
// but the parser doesn't know which one to choose. So the parser
// parses both, and remembers the results of both. Hir will choose
// one, and throw errors, if there is.
#[derive(Clone, Debug)]
pub struct DecoratorArg {
    pub keyword: Option<(InternedString, Span)>,
    pub expr: Result<Expr, Vec<Error>>,
    pub r#type: Result<Type, Vec<Error>>,
}

impl DecoratorArg {
    pub fn error_span_narrow(&self) -> Span {
        let mut span = Span::None;

        if let Some((_, name_span)) = self.keyword {
            span = name_span;
        }

        // It'd be okay even if both `expr` and `type` have spans.
        // Merging them would just choose the wider one.
        if let Ok(expr) = &self.expr {
            span = span.merge(expr.error_span_narrow());
        }

        if let Ok(r#type) = &self.r#type {
            span = span.merge(r#type.error_span_narrow());
        }

        span
    }

    pub fn error_span_wide(&self) -> Span {
        let mut span = Span::None;

        if let Some((_, name_span)) = self.keyword {
            span = name_span;
        }

        // It'd be okay even if both `expr` and `type` have spans.
        // Merging them would just choose the wider one.
        if let Ok(expr) = &self.expr {
            span = span.merge(expr.error_span_wide());
        }

        if let Ok(r#type) = &self.r#type {
            span = span.merge(r#type.error_span_wide());
        }

        span
    }
}

#[derive(Clone, Debug)]
pub struct Visibility {
    pub keyword_span: Span,

    // `pub` and `pub()` are different!
    pub args: Option<Vec<(InternedString, Span)>>,
    pub arg_group_span: Option<Span>,
}

impl<'t, 's> Tokens<'t, 's> {
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

                        if top_level && !*top_level_ {
                            break;
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

                    else if top_level && !top_level_ {
                        break;
                    }

                    let group_span = *span;
                    let mut tokens = Tokens::new(tokens, group_span.end(), &self.intermediate_dir);

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
                let mut tokens = Tokens::new(tokens, group_span.end(), &self.intermediate_dir);
                let args = tokens.parse_decorator_args()?;
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

    pub fn parse_decorator_args(&mut self) -> Result<Vec<DecoratorArg>, Vec<Error>> {
        let mut args = vec![];

        if self.is_empty() {
            return Ok(args);
        }

        loop {
            let keyword = match self.peek2() {
                (
                    Some(Token { kind: TokenKind::Ident(id), span }),
                    Some(Token { kind: TokenKind::Punct(Punct::Assign), .. }),
                ) => {
                    let (id, span) = (*id, *span);
                    self.cursor += 2;

                    Some((id, span))
                },
                _ => None,
            };

            let cursor = self.cursor;
            let arg_expr = self.parse_expr();
            let expr_end_cursor = self.cursor;
            let is_expr_err = arg_expr.is_err();
            self.cursor = cursor;
            let arg_type = self.parse_type();
            let type_end_cursor = self.cursor;
            let is_type_err = arg_type.is_err();

            // It's very tricky part. After parsing the argument, the cursor has to point to
            // the begin of the next argument (or eof).
            // The argument may be an expr or a type annotation, but we don't know that yet.
            //
            // But the good news is that, 1) the parser expects a comma or eof after parsing
            // an argument and 2) a comma is not a valid expr/type. So, it just moves the
            // cursor as far as possible.
            self.cursor = expr_end_cursor.max(type_end_cursor);

            args.push(DecoratorArg {
                keyword,
                expr: arg_expr,
                r#type: arg_type,
            });

            // We are not sure which error to throw, so we don't throw any.
            // But the errors are in `args`, so hir will throw the appropriate errors.
            if is_expr_err && is_type_err {
                return Ok(args);
            }

            match self.peek2() {
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), Some(_)) => {
                    self.cursor += 1;
                },
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) => {
                    return Ok(args);
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::Punct(Punct::Comma),
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
                (None, _) => {
                    return Ok(args);
                },
            }
        }
    }
}
