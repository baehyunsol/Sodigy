use sodigy_keyword::Keyword;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Token, TokenKind};
use crate::ParseSession;

#[derive(Debug)]
pub struct Let {
    name: InternedString,
    name_span: Span,
}

impl<'t> ParseSession<'t> {
    // KEYWORD_LET IDENTIFIER (PUNCT_COLON TY_EXPR) PUNCT_EQ EXPR PUNCT_SEMICOLON
    pub(crate) fn parse_let(&mut self) -> Result<Let, ()> {
        self.match_and_step(TokenKind::Keyword(Keyword::Let))?;
        let name = self.match_and_step(TokenKind::Identifier(InternedString::dummy()))?;
        let (name, name_span) = match name {
            Token {
                kind: TokenKind::Identifier(name),
                span,
            } => (*name, *span),
            _ => unreachable!(),
        };

        todo!();

        Ok(Let {
            name,
            name_span,
        })
    }
}
