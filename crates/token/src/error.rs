use crate::TokenKind;

// TokenKind for error variants
#[derive(Clone)]
pub enum ErrorToken {
    Any,
    Character(u8),
    Declaration,
}

impl From<&TokenKind> for ErrorToken {
    fn from(t: &TokenKind) -> ErrorToken {
        todo!()
    }
}
