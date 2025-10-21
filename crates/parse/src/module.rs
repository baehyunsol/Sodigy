use crate::Tokens;
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Keyword, Punct, TokenKind};

#[derive(Clone, Debug)]
pub struct Module {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
}

impl<'t> Tokens<'t> {
    pub fn parse_module(&mut self) -> Result<Module, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Struct))?.span;
        let (name, name_span) = self.pop_name_and_span()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Module {
            keyword_span,
            name,
            name_span,
        })
    }
}
