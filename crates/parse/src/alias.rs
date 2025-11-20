use crate::{Attribute, Generic, Tokens, Type};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Alias {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub group_span: Option<Span>,
    pub r#type: Type,
    pub attribute: Attribute,
}

impl<'t> Tokens<'t> {
    pub fn parse_alias(&mut self) -> Result<Alias, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Type))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        let mut generics = vec![];
        let mut group_span = None;

        match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Lt), span }) => {
                let mut group_span_ = *span;
                self.cursor += 1;
                generics = self.parse_generic_defs()?;
                group_span_ = group_span_.merge(self.match_and_pop(TokenKind::Punct(Punct::Gt))?.span);
                group_span = Some(group_span_);
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
            group_span,
            r#type,
            attribute: Attribute::new(),
        })
    }
}
