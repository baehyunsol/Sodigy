use crate::{
    Attribute,
    Generic,
    StructField,
    Tokens,
    Type,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
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
pub struct Enum {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub generic_group_span: Option<Span>,
    pub variants: Vec<EnumVariant>,
    pub attribute: Attribute,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: InternedString,
    pub name_span: Span,
    pub fields: EnumVariantFields,
    pub attribute: Attribute,
}

#[derive(Clone, Debug)]
pub enum EnumVariantFields {
    None,
    Tuple(Vec<(Type, Attribute)>),
    Struct(Vec<StructField>),
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_enum(&mut self) -> Result<Enum, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Enum))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        let mut generics = vec![];
        let mut generic_group_span = None;

        if let Some(Token { kind: TokenKind::Punct(Punct::Lt), span }) = self.peek() {
            generic_group_span = Some(*span);
            self.cursor += 1;
            generics = self.parse_generic_defs()?;
            let generic_span_end = self.match_and_pop(TokenKind::Punct(Punct::Gt))?.span;
            generic_group_span = generic_group_span.map(|span| span.merge(generic_span_end));
        }

        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;

        let Token {
            kind: TokenKind::Group {
                tokens: enum_body_tokens,
                ..
            },
            span: enum_body_span,
        } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
        let mut enum_body_tokens = Tokens::new(enum_body_tokens, enum_body_span.end(), &self.intermediate_dir);
        let variants = enum_body_tokens.parse_enum_variants()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Enum {
            keyword_span,
            name,
            name_span,
            generics,
            generic_group_span,
            variants,
            attribute: Attribute::new(),
        })
    }

    pub fn parse_enum_variants(&mut self) -> Result<Vec<EnumVariant>, Vec<Error>> {
        let mut variants = vec![];

        if self.peek().is_none() {
            return Ok(variants);
        }

        loop {
            let attribute = self.collect_attribute(false /* top_level */)?;
            let (name, name_span) = self.pop_name_and_span()?;

            match self.peek() {
                Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None => {
                    variants.push(EnumVariant {
                        name,
                        name_span,
                        fields: EnumVariantFields::None,
                        attribute,
                    });
                    self.cursor += 1;

                    if self.peek().is_none() {
                        break;
                    }
                },
                Some(Token { kind: TokenKind::Group { delim: Delim::Brace, tokens }, span }) => {
                    let mut struct_body_tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                    let fields = struct_body_tokens.parse_struct_fields()?;
                    variants.push(EnumVariant {
                        name,
                        name_span,
                        fields: EnumVariantFields::Struct(fields),
                        attribute,
                    });
                    self.cursor += 1;

                    match self.peek2() {
                        (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) | (None, _) => {
                            break;
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
                    }
                },
                Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }) => {
                    let mut tuple_body_tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);

                    if tuple_body_tokens.is_empty() {
                        variants.push(EnumVariant {
                            name,
                            name_span,
                            fields: EnumVariantFields::Tuple(vec![]),
                            attribute,
                        });
                    }

                    else {
                        let mut fields = vec![];

                        loop {
                            let attribute = tuple_body_tokens.collect_attribute(false /* top_level */)?;
                            let r#type = tuple_body_tokens.parse_type()?;
                            fields.push((r#type, attribute));

                            match tuple_body_tokens.peek2() {
                                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) | (None, _) => {
                                    break;
                                },
                                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), _) => {
                                    tuple_body_tokens.cursor += 1;
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
                            }
                        }

                        variants.push(EnumVariant {
                            name,
                            name_span,
                            fields: EnumVariantFields::Tuple(fields),
                            attribute,
                        });
                    }

                    self.cursor += 1;

                    match self.peek2() {
                        (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) | (None, _) => {
                            break;
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
                    }
                },
                Some(t) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::BraceOrCommaOrParenthesis,
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
            }
        }

        Ok(variants)
    }
}
