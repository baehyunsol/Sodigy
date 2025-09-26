use crate::{Decorator, DocComment, Expr, Tokens};
use sodigy_error::{Error, ErrorKind};
use sodigy_keyword::Keyword;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, ErrorToken, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Func {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<FuncArgDef>,
    pub r#type: Option<Expr>,
    pub value: Expr,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

#[derive(Clone, Debug)]
pub struct FuncArgDef {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub default_value: Option<Expr>,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

#[derive(Clone, Debug)]
pub struct CallArg {
    pub keyword: Option<(InternedString, Span)>,
    pub arg: Expr,
}

impl<'t> Tokens<'t> {
    // `func foo(x) = 3;`
    // `func bar(x: Int, y: Int): Int = x + y;`
    pub fn parse_func(&mut self) -> Result<Func, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Func))?.span;
        let (name, name_span) = self.pop_name_and_span()?;

        let arg_tokens = self.match_and_pop(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] })?;
        let arg_tokens_inner = match &arg_tokens.kind {
            TokenKind::Group { tokens, .. } => tokens,
            _ => unreachable!(),
        };
        let mut arg_tokens = Tokens::new(arg_tokens_inner, arg_tokens.span.end());
        let args = arg_tokens.parse_func_arg_defs()?;

        let r#type = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Colon), ..}) => {
                self.cursor += 1;
                Some(self.parse_expr()?)
            },
            _ => None,
        };

        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let value = self.parse_expr()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Func {
            keyword_span,
            name,
            name_span,
            args,
            r#type,
            value,

            // Its parent will set these fields.
            doc_comment: None,
            decorators: vec![],
        })
    }

    pub fn parse_func_arg_defs(&mut self) -> Result<Vec<FuncArgDef>, Vec<Error>> {
        let mut args = vec![];

        if self.peek().is_none() {
            return Ok(args);
        }

        'args: loop {
            let (doc_comment, decorators) = self.collect_doc_comment_and_decorators()?;
            let (name, name_span) = self.pop_name_and_span()?;
            let mut r#type = None;
            let mut default_value = None;

            'colon_or_value_or_comma: loop {
                match self.peek() {
                    Some(Token { kind: TokenKind::Punct(Punct::Colon), span }) => {
                        if r#type.is_some() {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Punct(Punct::Comma),
                                    got: ErrorToken::Punct(Punct::Colon),
                                },
                                span: *span,
                                ..Error::default()
                            }]);
                        }

                        self.cursor += 1;
                        r#type = Some(self.parse_expr()?);
                        continue 'colon_or_value_or_comma;
                    },
                    Some(Token { kind: TokenKind::Punct(Punct::Assign), span }) => {
                        if default_value.is_some() {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Punct(Punct::Comma),
                                    got: ErrorToken::Punct(Punct::Assign),
                                },
                                span: *span,
                                ..Error::default()
                            }]);
                        }

                        self.cursor += 1;
                        default_value = Some(self.parse_expr()?);
                        continue 'colon_or_value_or_comma;
                    },
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None => {
                        args.push(FuncArgDef {
                            name,
                            name_span,
                            r#type,
                            default_value,
                            doc_comment,
                            decorators,
                        });

                        match self.tokens.get(self.cursor + 1) {
                            Some(_) => {
                                self.cursor += 1;
                                continue 'args;
                            },
                            None => {
                                break 'args;
                            },
                        }
                    },
                    Some(t) => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::ColonOrComma,
                                got: (&t.kind).into(),
                            },
                            span: t.span,
                            ..Error::default()
                        }]);
                    },
                }
            }
        }

        Ok(args)
    }

    // (3, 4, x = 4, y = 5)
    pub fn parse_call_args(&mut self) -> Result<Vec<CallArg>, Vec<Error>> {
        let mut call_args = vec![];

        if self.is_empty() {
            return Ok(call_args);
        }

        loop {
            let keyword = match self.peek2() {
                (
                    Some(Token { kind: TokenKind::Identifier(id), span }),
                    Some(Token { kind: TokenKind::Punct(Punct::Assign), .. }),
                ) => Some((*id, *span)),
                _ => None,
            };
            let arg = self.parse_expr()?;
            call_args.push(CallArg { keyword, arg });

            match self.peek2() {
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), Some(_)) => {
                    self.cursor += 1;
                },
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) => {
                    return Ok(call_args);
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::Punct(Punct::Comma),
                            got: (&t.kind).into(),
                        },
                        span: t.span,
                        ..Error::default()
                    }]);
                },
                (None, _) => {
                    return Ok(call_args);
                },
            }
        }
    }
}
