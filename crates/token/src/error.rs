use crate::{Delim, Punct, TokenKind};
use sodigy_keyword::Keyword;

// TokenKind for error variants
#[derive(Clone, Debug)]
pub enum ErrorToken {
    Any,
    Character(u8),  // specific character
    Char,  // any character (in a character literal)
    Keyword(Keyword),
    Punct(Punct),
    Group(Delim),
    Identifier,
    Number,
    Declaration,
    Expr,
    Block,
    ColonOrComma,
}

impl From<&TokenKind> for ErrorToken {
    fn from(t: &TokenKind) -> ErrorToken {
        match t {
            TokenKind::Keyword(k) => ErrorToken::Keyword(*k),
            TokenKind::Punct(p) => ErrorToken::Punct(*p),
            TokenKind::Identifier(_) => ErrorToken::Identifier,
            TokenKind::Number(_) => ErrorToken::Number,
            TokenKind::Group { delim, .. } => ErrorToken::Group(*delim),
            _ => panic!("TODO: {t:?}"),
        }
    }
}
