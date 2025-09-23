use crate::TokenKind;

// TokenKind for error variants
#[derive(Clone, Debug)]
pub enum ErrorToken {
    Any,
    Character(u8),
    Identifier,
    Declaration,
    Expr,
    ColonOrComma,
    Comma,
}

impl From<&TokenKind> for ErrorToken {
    fn from(t: &TokenKind) -> ErrorToken {
        todo!()
    }
}
