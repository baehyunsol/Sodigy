use crate::session::InternedString;
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Clone)]
pub struct ModulePath (Vec<InternedString>);

impl ModulePath {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn tokens(&self, span: Span) -> Vec<Token> {
        self.0.iter().map(
            |s| Token {
                span,
                kind: TokenKind::Identifier(*s),
            }
        ).collect()
    }

}