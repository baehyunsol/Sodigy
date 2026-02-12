use crate::{Attribute, Field, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Use {
    pub keyword_span: Span,

    // use <full_path> as <name>
    pub name: InternedString,

    // if it's `use a.b.c;`, the span points to `c`.
    // if it's `use a.b.c as d;`, the span points to `d`.
    pub name_span: Span,

    pub full_path: Vec<Field>,
    pub attribute: Attribute,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_use(&mut self) -> Result<Vec<Use>, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Use))?.span;
        let mut uses = self.parse_use_recurse(false)?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        for r#use in uses.iter_mut() {
            r#use.keyword_span = keyword_span;
        }

        Ok(uses)
    }

    // It doesn't pop a semicolon.
    fn parse_use_recurse(
        &mut self,
        inside_group: bool,
    ) -> Result<Vec<Use>, Vec<Error>> {
        let mut prefix = vec![];
        let mut result = vec![];
        let mut dot_span = Span::None;

        loop {
            // `use a.{b`
            // `use a`
            let (name, name_span) = self.pop_name_and_span(false /* allow_wildcard */)?;
            prefix.push(Field::Name {
                name,
                name_span,
                dot_span,
                is_from_alias: false,
            });

            match self.peek() {
                Some(Token { kind: TokenKind::Punct(p), span }) => match p {
                    // `use a.{b.`
                    // `use a.`
                    Punct::Dot => {
                        dot_span = *span;
                        self.cursor += 1;

                        match self.peek() {
                            // `use a.b`
                            Some(Token { kind: TokenKind::Ident(_), .. }) => {
                                continue;
                            },
                            // `use a.{`
                            Some(Token { kind: TokenKind::Group { delim: Delim::Brace, tokens }, span }) => {
                                let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                                let mut inner = tokens.parse_use_recurse(true)?;
                                self.cursor += 1;

                                for u in inner.iter_mut() {
                                    u.full_path = vec![
                                        prefix.clone(),
                                        u.full_path.clone(),
                                    ].concat();
                                }

                                result.extend(inner);

                                match self.peek() {
                                    Some(Token { kind: TokenKind::Punct(p), span }) => match p {
                                        // `use a.{b.{c, d},` (valid)
                                        // `use a.{b, c},` (invalid)
                                        Punct::Comma => {
                                            if inside_group {
                                                self.cursor += 1;
                                                prefix = vec![];
                                            }

                                            else {
                                                return Err(vec![Error {
                                                    kind: ErrorKind::UnexpectedToken {
                                                        expected: ErrorToken::Punct(Punct::Semicolon),
                                                        got: ErrorToken::Punct(Punct::Comma),
                                                    },
                                                    spans: span.simple_error(),
                                                    note: Some(String::from("If you want to import multiple names, use another `use` statement.")),
                                                }]);
                                            }
                                        },
                                        // `use a.{b.{c, d};` (invalid)
                                        // `use a.{b, c};` (valid)
                                        Punct::Semicolon => {
                                            if inside_group {
                                                return Err(vec![Error {
                                                    kind: ErrorKind::UnexpectedToken {
                                                        expected: ErrorToken::Punct(Punct::Comma),
                                                        got: ErrorToken::Punct(Punct::Semicolon),
                                                    },
                                                    spans: span.simple_error(),
                                                    note: None,
                                                }]);
                                            }

                                            else {
                                                return Ok(result);
                                            }
                                        },
                                        _ => {
                                            let expected_punct = if inside_group { Punct::Comma } else { Punct::Semicolon };
                                            return Err(vec![Error {
                                                kind: ErrorKind::UnexpectedToken {
                                                    expected: ErrorToken::Punct(expected_punct),
                                                    got: ErrorToken::Punct(*p),
                                                },
                                                spans: span.simple_error(),
                                                note: None,
                                            }]);
                                        },
                                    },
                                    Some(t) => {
                                        let expected_punct = if inside_group { Punct::Comma } else { Punct::Semicolon };
                                        return Err(vec![Error {
                                            kind: ErrorKind::UnexpectedToken {
                                                expected: ErrorToken::Punct(expected_punct),
                                                got: (&t.kind).into(),
                                            },
                                            spans: t.span.simple_error(),
                                            note: None,
                                        }]);
                                    },
                                    // `use a.{b, c.{d, e}}` (valid)
                                    // `use a.{b, c}` (invalid, but `parse_use` will create an error)
                                    None => {
                                        return Ok(result);
                                    },
                                }
                            },
                            Some(t) => {
                                return Err(vec![Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected: ErrorToken::Ident,
                                        got: (&t.kind).into(),
                                    },
                                    spans: t.span.simple_error(),
                                    note: None,
                                }]);
                            },
                            // `use a.` (invalid)
                            None => {
                                return Err(vec![self.unexpected_end(ErrorToken::Ident)]);
                            },
                        }
                    },
                    // `use a.{b,` (valid)
                    // `use a,` (invalid)
                    Punct::Comma => {
                        if inside_group {
                            result.push(Use {
                                full_path: prefix.clone(),
                                name: prefix.last().unwrap().unwrap_name(),
                                name_span: prefix.last().unwrap().unwrap_name_span(),

                                // not available yet
                                keyword_span: Span::None,
                                attribute: Attribute::new(),
                            });
                            prefix = vec![];
                            self.cursor += 1;
                        }

                        else {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::DotOrSemicolon,
                                    got: ErrorToken::Punct(Punct::Semicolon),
                                },
                                spans: span.simple_error(),
                                note: Some(String::from("If you want to import multiple names, use another `use` statement.")),
                            }]);
                        }
                    },
                    // `use a.{b;` (invalid)
                    // `use a;` (valid)
                    Punct::Semicolon => {
                        if inside_group {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::CommaOrDot,
                                    got: ErrorToken::Punct(Punct::Semicolon),
                                },
                                spans: span.simple_error(),
                                note: None,
                            }]);
                        }

                        else {
                            result.push(Use {
                                full_path: prefix.clone(),
                                name: prefix.last().unwrap().unwrap_name(),
                                name_span: prefix.last().unwrap().unwrap_name_span(),

                                // not available yet
                                keyword_span: Span::None,
                                attribute: Attribute::new(),
                            });
                            return Ok(result);
                        }
                    },
                    _ => {
                        let expected_token = if inside_group { ErrorToken::CommaOrDot } else { ErrorToken::DotOrSemicolon };
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: expected_token,
                                got: ErrorToken::Punct(*p),
                            },
                            spans: span.simple_error(),
                            note: None,
                        }]);
                    },
                },
                // `use a as`
                Some(Token { kind: TokenKind::Keyword(Keyword::As), .. }) => {
                    self.cursor += 1;

                    // `use a as b`
                    let alias = self.pop_name_and_span(false /* allow_wildcard */)?;
                    result.push(Use {
                        full_path: prefix.clone(),
                        name: alias.0,
                        name_span: alias.1,

                        // not available yet
                        keyword_span: Span::None,
                        attribute: Attribute::new(),
                    });

                    match self.peek() {
                        Some(Token { kind: TokenKind::Punct(p), span }) => match p {
                            // `use a.{b as c,` (valid)
                            // `use a as b,` (invalid)
                            Punct::Comma => {
                                if inside_group {
                                    self.cursor += 1;
                                    prefix = vec![];
                                    continue;
                                }

                                else {
                                    return Err(vec![Error {
                                        kind: ErrorKind::UnexpectedToken {
                                            expected: ErrorToken::Punct(Punct::Semicolon),
                                            got: ErrorToken::Punct(Punct::Comma),
                                        },
                                        spans: span.simple_error(),
                                        note: Some(String::from("If you want to import multiple names, use another `use` statement.")),
                                    }]);
                                }
                            },
                            // `use a.{b as c;` (invalid)
                            // `use a as b;` (valid)
                            Punct::Semicolon => {
                                if inside_group {
                                    return Err(vec![Error {
                                        kind: ErrorKind::UnexpectedToken {
                                            expected: ErrorToken::Punct(Punct::Comma),
                                            got: ErrorToken::Punct(Punct::Semicolon),
                                        },
                                        spans: span.simple_error(),
                                        note: None,
                                    }]);
                                }

                                else {
                                    return Ok(result);
                                }
                            },
                            _ => {
                                let expected_punct = if inside_group { Punct::Comma } else { Punct::Semicolon };
                                return Err(vec![Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected: ErrorToken::Punct(expected_punct),
                                        got: ErrorToken::Punct(*p),
                                    },
                                    spans: span.simple_error(),
                                    note: None,
                                }]);
                            },
                        },
                        Some(t) => {
                            let expected_punct = if inside_group { Punct::Comma } else { Punct::Semicolon };
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Punct(expected_punct),
                                    got: (&t.kind).into(),
                                },
                                spans: t.span.simple_error(),
                                note: None,
                            }]);
                        },
                        // `use a.{b as c}` (valid)
                        // `use a as b` (invalid, but `parse_use` will create an error)
                        None => {
                            return Ok(result);
                        },
                    }
                },
                Some(t) => {
                    let expected_token = if inside_group { ErrorToken::CommaOrDot } else { ErrorToken::DotOrSemicolon };
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: expected_token,
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
                // `use a.{b, c}` (valid)
                // `use a` (invalid, but `parse_use` will create an error)
                None => {
                    result.push(Use {
                        full_path: prefix.clone(),
                        name: prefix.last().unwrap().unwrap_name(),
                        name_span: prefix.last().unwrap().unwrap_name_span(),

                        // not available yet
                        keyword_span: Span::None,
                        attribute: Attribute::new(),
                    });
                    return Ok(result);
                },
            }
        }
    }
}
