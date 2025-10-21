use crate::{Attribute, Tokens, Type};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Alias {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<(InternedString, Span)>,
    pub r#type: Type,
    pub attribute: Attribute,
}

impl<'t> Tokens<'t> {
    pub fn parse_alias(&mut self) -> Result<Alias, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Type))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        let mut args = vec![];

        match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Lt), span }) => {
                self.cursor += 1;

                loop {
                    match self.peek() {
                        Some(Token { kind: TokenKind::Identifier(id), span }) => {
                            args.push((*id, *span));
                            self.cursor += 1;

                            match self.peek() {
                                Some(Token { kind: TokenKind::Punct(Punct::Comma), span }) => {
                                    self.cursor += 1;
                                },
                                Some(Token { kind: TokenKind::Punct(Punct::Gt), .. }) => {
                                    self.cursor += 1;
                                    break;
                                },
                                _ => {},
                            }
                        },
                        Some(Token { kind: TokenKind::Punct(Punct::Gt), .. }) => {
                            self.cursor += 1;
                            break;
                        },
                        Some(t) => {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::CommaOrGt,
                                    got: (&t.kind).into(),
                                },
                                span: t.span,
                                ..Error::default()
                            }]);
                        },
                        None => {
                            return Err(vec![self.unexpected_end(ErrorToken::CommaOrGt)]);
                        },
                    }
                }
            },
            Some(Token { kind: TokenKind::Punct(Punct::Assign), .. }) => {
                self.cursor += 1;
            },
            Some(t) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::AssignOrLt,
                        got: (&t.kind).into(),
                    },
                    span: t.span,
                    ..Error::default()
                }]);
            },
            None => {
                return Err(vec![self.unexpected_end(ErrorToken::AssignOrLt)]);
            },
        }

        let r#type = self.parse_type()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Alias {
            keyword_span,
            name,
            name_span,
            args,
            r#type,
            attribute: Attribute::new(),
        })
    }
}
