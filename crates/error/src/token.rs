use sodigy_keyword::Keyword;
use sodigy_token::{Delim, Punct, TokenKind};

// TokenKind for error variants
#[derive(Clone, Debug)]
pub enum ErrorToken {
    Nothing,
    Any,
    Character(u8),  // specific character
    Char,  // any character (in a character literal)
    Keyword(Keyword),
    Punct(Punct),
    Group(Delim),
    Identifier,
    Number,
    TypeAnnotation,
    Declaration,
    Expr,
    Block,
    ColonOrComma,
    CommaOrGt,
    ParenthesisOrBrace,
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
