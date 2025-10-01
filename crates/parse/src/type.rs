use crate::Tokens;
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, ErrorToken, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct GenericDef;

impl<'t> Tokens<'t> {
    pub fn parse_generic_def(&mut self) -> Result<GenericDef, Vec<Error>> {
        todo!()
    }

    pub fn parse_generic_defs(&mut self) -> Result<Vec<GenericDef>, Vec<Error>> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum Type {
    Identifier {  // Int
        id: InternedString,
        span: Span,
    },
    Path(Vec<(InternedString, Span)>),  // module_name.struct_name
    Generic {  // Message<T>, Result<[Int], Error>
        r#type: Box<Type>,  // either `Type::Identifier` or `Type::Path`
        types: Vec<Type>,
    },
    Tuple {  // (Int, Int)
        types: Vec<Type>,
        group_span: Span,
    },
    List {  // [Int]
        r#type: Box<Type>,
        group_span: Span,
    },
    Func {  // Fn<(Int, Int): Int>
        // It's either `Type::Identifier` or `Type::Path`.
        // It's very likely to be `Type::Identifier("Fn")`
        r#type: Box<Type>,
        args: Vec<Type>,
        r#return: Box<Type>,
    },
}

impl Type {
    pub fn error_span(&self) -> Span {
        match self {
            Type::Identifier { span, .. } => *span,
            Type::Path(names) => names[0].1,
            Type::Generic { r#types, .. } => r#types[0].error_span(),
            Type::Tuple { group_span, .. } => *group_span,
            Type::List { group_span, .. } => *group_span,
            Type::Func { r#type, .. } => r#type.error_span(),
        }
    }
}

impl<'t> Tokens<'t> {
    pub fn parse_type(&mut self) -> Result<Type, Vec<Error>> {
        match self.peek2() {
            (
                Some(Token { kind: TokenKind::Identifier(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Dot), .. }),
            ) => {
                let mut path = vec![(*id, *span)];
                self.cursor += 2;

                loop {
                    match self.peek2() {
                        (
                            Some(Token { kind: TokenKind::Identifier(id), span }),
                            Some(Token { kind: TokenKind::Punct(Punct::Dot), .. }),
                        ) => {
                            path.push((*id, *span));
                            self.cursor += 2;
                        },
                        (
                            Some(Token { kind: TokenKind::Identifier(id), span }),
                            Some(Token { kind: TokenKind::Punct(Punct::Lt), .. }),
                        ) => {
                            path.push((*id, *span));

                            match self.try_parse_fn_type() {
                                Some(Ok((args, r#return))) => {
                                    self.match_and_pop(TokenKind::Punct(Punct::Gt))?;
                                    return Ok(Type::Func {
                                        r#type: Box::new(Type::Path(path)),
                                        args,
                                        r#return: Box::new(r#return),
                                    });
                                },
                                Some(Err(e)) => {
                                    return Err(e);
                                },
                                None => {
                                    let types = self.parse_types(StopAt::AngleBracket)?;
                                    self.match_and_pop(TokenKind::Punct(Punct::Gt))?;

                                    return Ok(Type::Generic {
                                        r#type: Box::new(Type::Path(path)),
                                        types,
                                    });
                                },
                            }
                        },
                        (Some(Token { kind: TokenKind::Identifier(id), span }), _) => {
                            path.push((*id, *span));
                            self.cursor += 1;
                            return Ok(Type::Path(path));
                        },
                        (Some(_), _) => todo!(),
                        (None, _) => todo!(),
                    }
                }
            },
            (
                Some(Token { kind: TokenKind::Identifier(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Lt), .. }),
            ) => {
                let (id, span) = (*id, *span);
                self.cursor += 2;

                match self.try_parse_fn_type() {
                    Some(Ok((args, r#return))) => {
                        self.match_and_pop(TokenKind::Punct(Punct::Gt))?;
                        Ok(Type::Func {
                            r#type: Box::new(Type::Identifier { id, span }),
                            args,
                            r#return: Box::new(r#return),
                        })
                    },
                    Some(Err(e)) => {
                        return Err(e);
                    },
                    None => {
                        let types = self.parse_types(StopAt::AngleBracket)?;
                        self.match_and_pop(TokenKind::Punct(Punct::Gt))?;

                        Ok(Type::Generic {
                            r#type: Box::new(Type::Identifier { id, span }),
                            types,
                        })
                    },
                }
            },
            (Some(Token { kind: TokenKind::Identifier(id), span }), _) => {
                let (id, span) = (*id, *span);
                self.cursor += 1;
                Ok(Type::Identifier { id, span })
            },
            (Some(Token { kind: TokenKind::Group { delim, tokens }, span }), _) => {
                let group_span = *span;
                let delim = *delim;
                let mut tokens = Tokens::new(tokens, group_span.end());

                let result = match delim {
                    Delim::Parenthesis => {
                        let types = tokens.parse_types(StopAt::Eof)?;
                        Ok(Type::Tuple {
                            types,
                            group_span,
                        })
                    },
                    Delim::Bracket => {
                        let r#type = tokens.parse_types(StopAt::Eof)?;

                        if let Some(unexpected_type_annotation) = r#type.get(1) {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Nothing,
                                    got: ErrorToken::TypeAnnotation,
                                },
                                span: unexpected_type_annotation.error_span(),
                                ..Error::default()
                            }]);
                        }

                        let r#type = Box::new(r#type[0].clone());
                        Ok(Type::List {
                            r#type,
                            group_span,
                        })
                    },
                    d => Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::TypeAnnotation,
                            got: ErrorToken::Group(d),
                        },
                        span: group_span,
                        ..Error::default()
                    }]),
                };

                self.cursor += 1;
                result
            },
            _ => todo!(),
        }
    }

    // If it's inside angle brackets, `self` contains the `>` token, so it must point to the `>`
    // after parsing is complete.
    // If it's inside parenthesis or square brackets, it must consume all the tokens.
    pub fn parse_types(&mut self, stop_at: StopAt) -> Result<Vec<Type>, Vec<Error>> {
        let mut types = vec![];
        let expected_token = match stop_at {
            // TODO: It'd be nice to have CommaOrParenthesis and CommaOrSquareBracket
            StopAt::Eof => ErrorToken::Punct(Punct::Comma),
            StopAt::AngleBracket => ErrorToken::CommaOrGt,
        };

        loop {
            // `self.parse_type` is called at least once because `types` cannot be empty.
            types.push(self.parse_type()?);

            match self.peek2() {
                // trailing comma
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    Some(Token { kind: TokenKind::Punct(Punct::Gt), span }),
                ) => match stop_at {
                    StopAt::Eof => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: expected_token,
                                got: ErrorToken::Punct(Punct::Gt),
                            },
                            span: *span,
                            ..Error::default()
                        }]);
                    },
                    StopAt::AngleBracket => {
                        return Ok(types);
                    },
                },
                // trailing comma
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) => match stop_at {
                    StopAt::Eof => {
                        return Ok(types);
                    },
                    StopAt::AngleBracket => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedEof {
                                expected: expected_token,
                            },
                            span: self.span_end,
                            ..Error::default()
                        }]);
                    },
                },
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), _) => {
                    self.cursor += 1;
                },
                (Some(Token { kind: TokenKind::Punct(Punct::Gt), span }), _) => match stop_at {
                    StopAt::Eof => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: expected_token,
                                got: ErrorToken::Punct(Punct::Gt),
                            },
                            span: *span,
                            ..Error::default()
                        }]);
                    },
                    StopAt::AngleBracket => {
                        return Ok(types);
                    },
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: expected_token,
                            got: (&t.kind).into(),
                        },
                        span: t.span,
                        ..Error::default()
                    }]);
                },
                (None, _) => match stop_at {
                    StopAt::Eof => {
                        return Ok(types);
                    },
                    StopAt::AngleBracket => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedEof {
                                expected: expected_token,
                            },
                            span: self.span_end,
                            ..Error::default()
                        }]);
                    },
                },
            }
        }
    }

    // It first checks if it looks like fn type signature or not.
    // If it's not, it's just a generic type signature, so it returns `None`.
    // If it is, it tries to parse the fn type signature and returns `Some(Ok(_))` or `Some(Err(_))`.
    // If it's successful, the cursor points to `>`.
    pub fn try_parse_fn_type(&mut self) -> Option<Result<(Vec<Type>, Type), Vec<Error>>> {
        match self.peek2() {
            (
                Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }),
                Some(Token { kind: TokenKind::Punct(Punct::Colon), .. }),
            ) => {
                let mut tokens = Tokens::new(tokens, span.end());
                let args = match tokens.parse_types(StopAt::Eof) {
                    Ok(args) => args,
                    Err(e) => {
                        return Some(Err(e));
                    },
                };

                self.cursor += 2;
                let r#return = match self.parse_type() {
                    Ok(r#return) => r#return,
                    Err(e) => {
                        return Some(Err(e));
                    },
                };

                Some(Ok((args, r#return)))
            },
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum StopAt {
    Eof,
    AngleBracket,
}
