use crate::span::Span;
use crate::token::TokenKind;

mod kind;
mod parse;
#[cfg(test)] mod tests;

pub use kind::ValueKind;
pub use parse::parse_value;

#[derive(Clone)]
pub struct Value {
    span: Span,
    kind: ValueKind
}

impl Value {

    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    pub fn get_first_token(&self) -> TokenKind {
        self.kind.get_first_token()
    }

}