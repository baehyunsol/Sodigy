use crate::{Attribute, Expr, Tokens};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_token::{Keyword, Punct, TokenKind};

#[derive(Clone, Debug)]
pub struct Do {
    pub keyword_span: Span,
    pub value: Expr,
    pub attribute: Attribute,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_do(&mut self) -> Result<Do, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Do))?.span.clone();
        let value = self.parse_expr(true)?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Do {
            keyword_span,
            value,
            attribute: Attribute::new(),
        })
    }
}
