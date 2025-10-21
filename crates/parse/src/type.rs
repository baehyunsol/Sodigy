use crate::{Field, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct GenericDef {
    pub name: InternedString,
    pub name_span: Span,
}

impl<'t> Tokens<'t> {
    pub fn parse_generic_def(&mut self) -> Result<GenericDef, Vec<Error>> {
        let (name, name_span) = self.pop_name_and_span()?;
        Ok(GenericDef {
            name,
            name_span,
        })
    }

    pub fn parse_generic_defs(&mut self) -> Result<Vec<GenericDef>, Vec<Error>> {
        let mut generics = vec![];

        loop {
            let generic = self.parse_generic_def()?;
            generics.push(generic);

            match self.peek2() {
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    Some(Token { kind: TokenKind::Punct(Punct::Gt), .. }),
                ) => {
                    self.cursor += 1;
                    return Ok(generics);
                },
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    _,
                ) => {
                    self.cursor += 1;
                },
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Gt), .. }),
                    _,
                ) => {
                    return Ok(generics);
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::CommaOrGt,
                            got: (&t.kind).into(),
                        },
                        span: t.span,
                        ..Error::default()
                    }]);
                },
                (None, _) => {
                    return Err(vec![self.unexpected_end(ErrorToken::CommaOrGt)]);
                },
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Type {
    // `Int`, `String`, `Bool`, `T` in `fn foo<T>()`
    Identifier {
        id: InternedString,
        span: Span,
    },
    // `module_name.StructName`
    Path {
        id: InternedString,
        id_span: Span,
        fields: Vec<Field>,
    },
    // `Message<T>`, `Result<[Int], Error>`
    Generic {
        r#type: Box<Type>,  // either `Type::Identifier` or `Type::Path`
        args: Vec<Type>,
        group_span: Span,
    },
    // `(Int, Int)`
    Tuple {
        types: Vec<Type>,
        group_span: Span,
    },
    // `[Int]`
    List {
        r#type: Box<Type>,
        group_span: Span,
    },
    Func {  // `Fn(Int, Int) -> Int`
        // It's either `Type::Identifier` or `Type::Path`.
        // It's very likely to be `Type::Identifier("Fn")`.
        // If it's not `Fn`, it's 99% an error, but I want to throw
        // errors at later step because that's more helpful to the users.
        r#type: Box<Type>,
        args: Vec<Type>,
        r#return: Box<Type>,
    },
    // `_` in `[_]`
    // It'll be infered, if possible.
    Wildcard(Span),
}

impl Type {
    pub fn error_span(&self) -> Span {
        match self {
            Type::Identifier { span, .. } |
            Type::Wildcard(span) => *span,
            Type::Path { fields, .. } => match fields.get(0) {
                Some(Field::Name { dot_span, .. }) => *dot_span,
                _ => unreachable!(),
            },
            Type::Generic { args, .. } => args[0].error_span(),
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
                Some(Token { kind: TokenKind::Punct(Punct::Dot), span: dot_span }),
            ) => {
                let mut path = vec![(*id, *span)];
                let mut dot_spans = vec![*dot_span];
                self.cursor += 2;

                loop {
                    match self.peek2() {
                        (
                            Some(Token { kind: TokenKind::Identifier(id), span }),
                            Some(Token { kind: TokenKind::Punct(Punct::Dot), span: dot_span }),
                        ) => {
                            path.push((*id, *span));
                            dot_spans.push(*dot_span);
                            self.cursor += 2;
                        },
                        (
                            Some(Token { kind: TokenKind::Identifier(id), span: span1 }),
                            Some(Token { kind: TokenKind::Punct(Punct::Lt), span: span2 }),
                        ) => {
                            let group_span_start = *span2;
                            path.push((*id, *span1));
                            let args = self.parse_types(StopAt::AngleBracket)?;
                            let group_span_end = self.match_and_pop(TokenKind::Punct(Punct::Gt))?.span;

                            return Ok(Type::Generic {
                                r#type: Box::new(Type::Path {
                                    id: path[0].0,
                                    id_span: path[0].1,
                                    fields: path[1..].iter().zip(dot_spans.iter()).map(
                                        |((id, id_span), dot_span)| Field::Name {
                                            name: *id,
                                            span: *id_span,
                                            dot_span: *dot_span,
                                        },
                                    ).collect(),
                                }),
                                args,
                                group_span: group_span_start.merge(group_span_end),
                            });
                        },
                        (
                            Some(Token { kind: TokenKind::Identifier(id), span }),
                            Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                        ) => todo!(),  // maybe func
                        (Some(Token { kind: TokenKind::Identifier(id), span }), _) => {
                            path.push((*id, *span));
                            self.cursor += 1;
                            return Ok(Type::Path {
                                id: path[0].0,
                                id_span: path[0].1,
                                fields: path[1..].iter().zip(dot_spans.iter()).map(
                                    |((id, id_span), dot_span)| Field::Name {
                                        name: *id,
                                        span: *id_span,
                                        dot_span: *dot_span,
                                    },
                                ).collect(),
                            });
                        },
                        (Some(_), _) => todo!(),
                        (None, _) => todo!(),
                    }
                }
            },
            (
                Some(Token { kind: TokenKind::Identifier(id), span: span1 }),
                Some(Token { kind: TokenKind::Punct(Punct::Lt), span: span2 }),
            ) => {
                let group_span_start = *span2;
                let (id, span) = (*id, *span1);
                self.cursor += 2;

                let args = self.parse_types(StopAt::AngleBracket)?;
                let group_span_end = self.match_and_pop(TokenKind::Punct(Punct::Gt))?.span;

                Ok(Type::Generic {
                    r#type: Box::new(Type::Identifier { id, span }),
                    args,
                    group_span: group_span_start.merge(group_span_end),
                })
            },
            // Fn(Int, Int) -> Int
            (
                Some(Token { kind: TokenKind::Identifier(id), span: span1 }),
                Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span: span2 }),
            ) => {
                let (name, name_span) = (*id, *span1);
                let mut arg_tokens = Tokens::new(tokens, span2.end());
                let args = arg_tokens.parse_types(StopAt::Eof)?;

                self.cursor += 2;
                self.match_and_pop(TokenKind::Punct(Punct::ReturnType))?;
                let r#return = self.parse_type()?;

                Ok(Type::Func {
                    r#type: Box::new(Type::Identifier {
                        id: name,
                        span: name_span,
                    }),
                    args,
                    r#return: Box::new(r#return),
                })
            },
            (Some(Token { kind: TokenKind::Identifier(id), span }), _) => {
                let (id, span) = (*id, *span);
                self.cursor += 1;

                match id.try_unintern_short_string() {
                    Some(id) if id == b"_" => Ok(Type::Wildcard(span)),
                    _ => Ok(Type::Identifier { id, span }),
                }
            },
            (Some(Token { kind: TokenKind::Group { delim, tokens }, span }), _) => {
                let group_span = *span;
                let delim = *delim;
                let mut tokens = Tokens::new(tokens, group_span.end());

                let result = match delim {
                    Delim::Parenthesis => {
                        if tokens.is_empty() {
                            Ok(Type::Tuple {
                                types: vec![],
                                group_span,
                            })
                        }

                        else {
                            let types = tokens.parse_types(StopAt::Eof)?;
                            Ok(Type::Tuple {
                                types,
                                group_span,
                            })
                        }
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
            ts => panic!("TODO: {ts:?}"),
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
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum StopAt {
    Eof,
    AngleBracket,
}
