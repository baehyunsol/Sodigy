use crate::{Field, Path, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Generic {
    pub name: InternedString,
    pub name_span: Span,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_generic_def(&mut self) -> Result<Generic, Vec<Error>> {
        let (name, name_span) = self.pop_name_and_span(false /* allow_wildcard */)?;
        Ok(Generic {
            name,
            name_span,
        })
    }

    pub fn parse_generic_defs(&mut self) -> Result<Vec<Generic>, Vec<Error>> {
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
                        spans: t.span.simple_error(),
                        note: None,
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
    // `module_name.StructName` is a path.
    // `Int` or `T` are also paths without any fields!
    Path(Path),
    // `Message<T>`, `Result<[Int], Error>`
    Param {
        constructor: Path,
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
        fn_constructor: Path,  // "Fn", "ImpureFn" or "PureFn".

        // of `(Int, Int)`
        group_span: Span,

        params: Vec<Type>,
        r#return: Box<Type>,
    },
    // `_` in `[_]`
    // It'll be infered, if possible.
    Wildcard(Span),

    // `!`
    // It's subtype of every type.
    Never(Span),
}

impl Type {
    pub fn error_span_narrow(&self) -> Span {
        match self {
            Type::Wildcard(span) |
            Type::Never(span) => span.clone(),
            Type::Path(path) |
            Type::Param { constructor: path, .. } |
            Type::Func { fn_constructor: path, .. } => path.error_span_narrow(),
            Type::Tuple { group_span, .. } => group_span.clone(),
            Type::List { group_span, .. } => group_span.clone(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Type::Path(path) => path.error_span_wide(),
            Type::Param { constructor, group_span, .. } => constructor.error_span_wide().merge(group_span),
            Type::Tuple { group_span, .. } => group_span.clone(),
            Type::List { group_span, .. } => group_span.clone(),
            Type::Func { fn_constructor, group_span, r#return, .. } => fn_constructor.error_span_wide()
                .merge(group_span)
                .merge(&r#return.error_span_wide()),
            Type::Wildcard(span) | Type::Never(span) => span.clone(),
        }
    }
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_type(&mut self) -> Result<Type, Vec<Error>> {
        match self.peek2() {
            (
                Some(Token { kind: TokenKind::Ident(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Dot), span: dot_span }),
            ) => {
                let mut path = vec![(*id, span.clone())];
                let mut dot_spans = vec![dot_span.clone()];
                self.cursor += 2;

                loop {
                    match self.peek2() {
                        (
                            Some(Token { kind: TokenKind::Ident(id), span }),
                            Some(Token { kind: TokenKind::Punct(Punct::Dot), span: dot_span }),
                        ) => {
                            path.push((*id, span.clone()));
                            dot_spans.push(dot_span.clone());
                            self.cursor += 2;
                        },
                        (
                            Some(Token { kind: TokenKind::Ident(id), span: span1 }),
                            Some(Token { kind: TokenKind::Punct(Punct::Lt), span: span2 }),
                        ) => {
                            let group_span_start = span2.clone();
                            path.push((*id, span1.clone()));
                            self.cursor += 1;
                            let (args, group_span_end) = self.parse_types_in_angle_brackets()?;

                            return Ok(Type::Param {
                                constructor: Path {
                                    id: path[0].0,
                                    id_span: path[0].1.clone(),
                                    fields: path[1..].iter().zip(dot_spans.iter()).map(
                                        |((id, id_span), dot_span)| Field::Name {
                                            name: *id,
                                            name_span: id_span.clone(),
                                            dot_span: dot_span.clone(),
                                            is_from_alias: false,
                                        },
                                    ).collect(),
                                    dotfish: vec![None; path.len()],
                                },
                                args,
                                group_span: group_span_start.merge(&group_span_end),
                            });
                        },
                        (
                            Some(Token { kind: TokenKind::Ident(id), span }),
                            Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                        ) => todo!(),  // maybe func
                        (Some(Token { kind: TokenKind::Ident(id), span }), _) => {
                            path.push((*id, span.clone()));
                            self.cursor += 1;
                            return Ok(Type::Path(Path {
                                id: path[0].0,
                                id_span: path[0].1.clone(),
                                fields: path[1..].iter().zip(dot_spans.iter()).map(
                                    |((id, id_span), dot_span)| Field::Name {
                                        name: *id,
                                        name_span: id_span.clone(),
                                        dot_span: dot_span.clone(),
                                        is_from_alias: false,
                                    },
                                ).collect(),
                                dotfish: vec![None; path.len()],
                            }));
                        },
                        (Some(t), _) => {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Ident,
                                    got: (&t.kind).into(),
                                },
                                spans: t.span.simple_error(),
                                note: None,
                            }]);
                        },
                        (None, _) => {
                            return Err(vec![self.unexpected_end(ErrorToken::Ident)]);
                        },
                    }
                }
            },
            (
                Some(Token { kind: TokenKind::Ident(id), span: span1 }),
                Some(Token { kind: TokenKind::Punct(Punct::Lt), span: span2 }),
            ) => {
                let group_span_start = span2.clone();
                let (id, id_span) = (*id, span1.clone());
                self.cursor += 1;
                let (args, group_span_end) = self.parse_types_in_angle_brackets()?;

                Ok(Type::Param {
                    constructor: Path {
                        id,
                        id_span,
                        fields: vec![],
                        dotfish: vec![None],
                    },
                    args,
                    group_span: group_span_start.merge(&group_span_end),
                })
            },
            // Fn(Int, Int) -> Int
            (
                Some(Token { kind: TokenKind::Ident(id), span: span1 }),
                Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span: span2 }),
            ) => {
                let (id, id_span) = (*id, span1.clone());
                let group_span = span2.clone();
                let mut param_tokens = Tokens::new(tokens, span2.end(), false, self.intermediate_dir);
                let params = param_tokens.parse_types()?;

                self.cursor += 2;
                self.match_and_pop(TokenKind::Punct(Punct::ReturnType))?;
                let r#return = self.parse_type()?;

                Ok(Type::Func {
                    fn_constructor: Path {
                        id,
                        id_span,
                        fields: vec![],
                        dotfish: vec![None],
                    },
                    group_span,
                    params,
                    r#return: Box::new(r#return),
                })
            },
            (Some(Token { kind: TokenKind::Ident(id), span }), _) => {
                let (id, id_span) = (*id, span.clone());
                self.cursor += 1;

                Ok(Type::Path(Path {
                    id,
                    id_span,
                    fields: vec![],

                    // no dotfish operators for type annotations
                    dotfish: vec![None],
                }))
            },
            (Some(Token { kind: TokenKind::Group { delim, tokens }, span }), _) => {
                let group_span = span.clone();
                let delim = *delim;
                let mut tokens = Tokens::new(tokens, group_span.end(), false, self.intermediate_dir);

                let result = match delim {
                    Delim::Parenthesis => {
                        let types = tokens.parse_types()?;
                        let mut is_tuple = types.len() != 1;

                        // `(Int)` is just an integer type, but `(Int,)` is a tuple type
                        if types.len() == 1 && matches!(
                            tokens.last(),
                            Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                        ) {
                            is_tuple = true;
                        }

                        if is_tuple {
                            Ok(Type::Tuple {
                                types,
                                group_span,
                            })
                        }

                        else {
                            Ok(types[0].clone())
                        }
                    },
                    Delim::Bracket => {
                        let r#type = tokens.parse_types()?;

                        if let Some(unexpected_type_annot) = r#type.get(1) {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Nothing,
                                    got: ErrorToken::TypeAnnot,
                                },
                                spans: unexpected_type_annot.error_span_wide().simple_error(),
                                note: None,
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
                            expected: ErrorToken::TypeAnnot,
                            got: ErrorToken::Group(d),
                        },
                        spans: group_span.simple_error(),
                        note: None,
                    }]),
                };

                self.cursor += 1;
                result
            },
            (Some(Token { kind: TokenKind::Punct(Punct::Factorial), span }), _) => {
                let result = Ok(Type::Never(span.clone()));
                self.cursor += 1;
                result
            },
            (Some(Token { kind: TokenKind::Wildcard, span }), _) => {
                let result = Ok(Type::Wildcard(span.clone()));
                self.cursor += 1;
                result
            },
            (Some(t), _) => Err(vec![Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::TypeAnnot,
                    got: (&t.kind).into(),
                },
                spans: t.span.simple_error(),
                note: None,
            }]),
            (None, _) => Err(vec![self.unexpected_end(ErrorToken::TypeAnnot)]),
        }
    }

    // When it's called the cursor is pointing to `<`.
    // When it returns successfully, the cursor must be pointing to the token
    // after the closing angle bracket. The closing angle bracket can be `>` or `>>`.
    //
    // This is tricky because the lexer treats `>>` as a single token. So it splits `>>`,
    // creates new `Tokens` instance, and parses the types with the new `Tokens`.
    //
    // There's an edge case: `x as <Int>>10`. It's a valid code, but the parser cannot handle this.
    // The user has to write `(x as <Int>)>10` or `x as <Int> > 10`.
    // We have to throw `AmbiguousAngleBrackets` error.
    pub fn parse_types_in_angle_brackets(&mut self) -> Result<(Vec<Type>, Span), Vec<Error>> {
        let mut new_tokens = vec![];
        self.match_and_pop(TokenKind::Punct(Punct::Lt))?;
        let mut stack: i32 = 1;
        let mut next_cursor = 0;
        let mut span_end = Span::None;

        for (cursor, token) in self.enumerate_forward() {
            if stack <= 0 {
                break;
            }

            match token {
                Token { kind: TokenKind::Punct(Punct::Lt), .. } => {
                    stack += 1;
                },
                Token { kind: TokenKind::Punct(Punct::Gt), span } => {
                    stack -= 1;
                    span_end = span.clone();
                    next_cursor = cursor;
                },
                Token { kind: TokenKind::Punct(Punct::Shr), span } => {
                    new_tokens.push(Token { kind: TokenKind::Punct(Punct::Gt), span: span.clone() });
                    new_tokens.push(Token { kind: TokenKind::Punct(Punct::Gt), span: span.clone() });
                    stack -= 2;
                    span_end = span.clone();
                    next_cursor = cursor;
                    continue;
                },
                _ => {},
            }

            new_tokens.push(token.clone());
        }

        match stack {
            0 => {},  // no problem
            1.. => {
                return Err(vec![self.unexpected_end(ErrorToken::Punct(Punct::Gt))]);
            },
            _ => {
                return Err(vec![Error {
                    kind: ErrorKind::AmbiguousAngleBrackets,
                    spans: span_end.simple_error(),
                    note: None,
                }]);
            },
        }

        // pops the last `>` (or `>>`) token
        new_tokens.pop().unwrap();

        if new_tokens.is_empty() {
            // `x as <>` is an error
            todo!();
        }

        self.cursor = next_cursor + 1;
        let mut new_tokens = Tokens::new(&new_tokens, span_end.clone(), false, self.intermediate_dir);
        Ok((new_tokens.parse_types()?, span_end))
    }

    // It must consume all the tokens.
    pub fn parse_types(&mut self) -> Result<Vec<Type>, Vec<Error>> {
        let mut types = vec![];

        if self.peek().is_none() {
            return Ok(vec![]);
        }

        loop {
            types.push(self.parse_type()?);

            match self.peek2() {
                // trailing comma
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) => {
                    return Ok(types);
                },
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), _) => {
                    self.cursor += 1;
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
                    return Ok(types);
                },
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Dotfish {
    pub types: Vec<Type>,
    pub group_span: Span,
}
