use crate::{Attribute, GenericDef, Tokens};
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
    pub generics: Vec<GenericDef>,
    pub variants: Vec<EnumVariantDef>,
    pub attribute: Attribute,
}

#[derive(Clone, Debug)]
pub struct EnumVariantDef {
    pub name: InternedString,
    pub name_span: Span,
    pub args: EnumVariantArgs,
    pub attribute: Attribute,
}

#[derive(Clone, Debug)]
pub enum EnumVariantArgs {
    None,

    // TODO: fields
    Tuple,
    Struct,
}

impl<'t> Tokens<'t> {
    pub fn parse_enum(&mut self) -> Result<Enum, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Enum))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        let mut generics = vec![];

        if let Some(Token { kind: TokenKind::Punct(Punct::Lt), .. }) = self.peek() {
            self.cursor += 1;
            generics = self.parse_generic_defs()?;
            self.match_and_pop(TokenKind::Punct(Punct::Gt))?;
        }

        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;

        let Token {
            kind: TokenKind::Group {
                tokens: enum_body_tokens,
                ..
            },
            span: enum_body_span,
        } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
        let mut enum_body_tokens = Tokens::new(enum_body_tokens, enum_body_span.end());
        let variants = enum_body_tokens.parse_enum_variants()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Enum {
            keyword_span,
            name,
            name_span,
            generics,
            variants,
            attribute: Attribute::new(),
        })
    }

    pub fn parse_enum_variants(&mut self) -> Result<Vec<EnumVariantDef>, Vec<Error>> {
        let mut variants = vec![];

        if self.peek().is_none() {
            return Ok(variants);
        }

        loop {
            let attribute = self.collect_attribute()?;
            let (name, name_span) = self.pop_name_and_span()?;

            match self.peek() {
                Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None => {
                    variants.push(EnumVariantDef {
                        name,
                        name_span,
                        args: EnumVariantArgs::None,
                        attribute,
                    });
                    self.cursor += 1;

                    if self.peek().is_none() {
                        break;
                    }
                },
                Some(Token { kind: TokenKind::Group { delim: Delim::Brace, tokens }, span }) => todo!(),
                Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }) => todo!(),
                Some(t) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::BraceOrCommaOrParenthesis,
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        ..Error::default()
                    }]);
                },
            }
        }

        Ok(variants)
    }
}
