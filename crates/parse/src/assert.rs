use crate::{Attribute, Expr, Tokens};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_token::{Keyword, Punct, TokenKind};

#[derive(Clone, Debug)]
pub struct Assert {
    pub keyword_span: Span,
    pub value: Box<Expr>,
    pub attribute: Attribute,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_assert(&mut self) -> Result<Assert, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Assert))?.span;
        let value = self.parse_expr()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Assert {
            keyword_span,
            value: Box::new(value),
            attribute: Attribute::new(),
        })
    }
}
