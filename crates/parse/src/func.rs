use crate::{
    Attribute,
    Expr,
    Generic,
    Tokens,
    Type,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Func {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub params: Vec<FuncParam>,
    pub r#type: Option<Type>,

    // A poly or built-in may not have a body.
    pub value: Option<Expr>,

    pub attribute: Attribute,
}

#[derive(Clone, Debug)]
pub struct FuncParam {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Type>,
    pub default_value: Option<Expr>,
    pub attribute: Attribute,
}

#[derive(Clone, Debug)]
pub struct CallArg {
    pub keyword: Option<(InternedString, Span)>,
    pub arg: Expr,
}

impl<'t> Tokens<'t> {
    // `fn foo(x) = 3;`
    // `fn bar(x: Int, y: Int): Int = x + y;`
    pub fn parse_func(&mut self) -> Result<Func, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Fn))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        let mut generics = vec![];

        if let Some(Token { kind: TokenKind::Punct(Punct::Lt), .. }) = self.peek() {
            self.cursor += 1;
            generics = self.parse_generic_defs()?;
            self.match_and_pop(TokenKind::Punct(Punct::Gt))?;
        }

        let param_tokens = self.match_and_pop(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] })?;
        let param_tokens_inner = match &param_tokens.kind {
            TokenKind::Group { tokens, .. } => tokens,
            _ => unreachable!(),
        };
        let mut param_tokens = Tokens::new(param_tokens_inner, param_tokens.span.end());
        let params = param_tokens.parse_func_params()?;

        let r#type = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::ReturnType), ..}) => {
                self.cursor += 1;
                Some(self.parse_type()?)
            },
            _ => None,
        };

        let value = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Assign), .. }) => {
                self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
                let value = Some(self.parse_expr()?);
                self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;
                value
            },
            Some(Token { kind: TokenKind::Punct(Punct::Semicolon), .. }) => {
                self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;
                None
            },
            Some(t) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::AssignOrSemicolon,
                        got: (&t.kind).into(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            None => {
                return Err(vec![self.unexpected_end(ErrorToken::AssignOrSemicolon)]);
            },
        };

        Ok(Func {
            keyword_span,
            name,
            name_span,
            generics,
            params,
            r#type,
            value,
            attribute: Attribute::new(),
        })
    }

    pub fn parse_func_params(&mut self) -> Result<Vec<FuncParam>, Vec<Error>> {
        let mut params = vec![];

        if self.peek().is_none() {
            return Ok(params);
        }

        'params: loop {
            let attribute = self.collect_attribute(false /* top_level */)?;
            let (name, name_span) = self.pop_name_and_span()?;
            let mut r#type = None;
            let mut default_value = None;
            let mut prev_colon_span = None;
            let mut prev_assignment_span = None;

            'colon_or_value_or_comma: loop {
                match self.peek() {
                    Some(Token { kind: TokenKind::Punct(Punct::Colon), span }) => {
                        let span = *span;

                        if r#type.is_some() {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Punct(Punct::Comma),
                                    got: ErrorToken::Punct(Punct::Colon),
                                },
                                spans: vec![
                                    RenderableSpan {
                                        span,
                                        auxiliary: false,
                                        note: None,
                                    },
                                    RenderableSpan {
                                        span: prev_colon_span.unwrap(),
                                        auxiliary: true,
                                        note: Some(String::from("We already have a type annotation here.")),
                                    },
                                ],
                                ..Error::default()
                            }]);
                        }

                        self.cursor += 1;
                        prev_colon_span = Some(span);
                        r#type = Some(self.parse_type()?);
                        continue 'colon_or_value_or_comma;
                    },
                    Some(Token { kind: TokenKind::Punct(Punct::Assign), span }) => {
                        let span = *span;

                        if default_value.is_some() {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Punct(Punct::Comma),
                                    got: ErrorToken::Punct(Punct::Assign),
                                },
                                spans: vec![
                                    RenderableSpan {
                                        span,
                                        auxiliary: false,
                                        note: None,
                                    },
                                    RenderableSpan {
                                        span: prev_assignment_span.unwrap(),
                                        auxiliary: true,
                                        note: Some(String::from("We already have a default value here.")),
                                    },
                                ],
                                note: None,
                            }]);
                        }

                        self.cursor += 1;
                        prev_assignment_span = Some(span);
                        default_value = Some(self.parse_expr()?);
                        continue 'colon_or_value_or_comma;
                    },
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None => {
                        params.push(FuncParam {
                            name,
                            name_span,
                            r#type,
                            default_value,
                            attribute,
                        });

                        match self.tokens.get(self.cursor + 1) {
                            Some(_) => {
                                self.cursor += 1;
                                continue 'params;
                            },
                            None => {
                                break 'params;
                            },
                        }
                    },
                    Some(t) => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::ColonOrComma,
                                got: (&t.kind).into(),
                            },
                            spans: t.span.simple_error(),
                            ..Error::default()
                        }]);
                    },
                }
            }
        }

        Ok(params)
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
                ) => {
                    let (id, span) = (*id, *span);
                    self.cursor += 2;

                    Some((id, span))
                },
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
                        spans: t.span.simple_error(),
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
