use crate::{Attribute, GenericDef, Tokens, Type};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Alias {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<GenericDef>,
    pub r#type: Type,
    pub attribute: Attribute,
}

impl<'t> Tokens<'t> {
    pub fn parse_alias(&mut self) -> Result<Alias, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Type))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        let mut generics = vec![];

        match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Lt), .. }) => {
                self.cursor += 1;
                generics = self.parse_generic_defs()?;
                self.match_and_pop(TokenKind::Punct(Punct::Gt))?;
            },
            _ => {},
        }

        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let r#type = self.parse_type()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Alias {
            keyword_span,
            name,
            name_span,
            generics,
            r#type,
            attribute: Attribute::new(),
        })
    }
}
