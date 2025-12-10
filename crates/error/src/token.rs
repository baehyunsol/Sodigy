use sodigy_token::{Delim, Keyword, Punct, TokenKind};

// TokenKind for error variants
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ErrorToken {
    Nothing,
    Any,
    Character(u8),  // specific character
    AnyCharacter,   // in a character literal
    Keyword(Keyword),
    Punct(Punct),
    Group(Delim),
    Identifier,
    Generic,
    Number,
    String,
    TypeAnnotation,
    Declaration,
    Expr,
    Path,  // and identifier or a path
    Pattern,
    Item,  // fn / struct / enum / use / type / let
    Block,
    Operator,
    AssignOrLt,
    AssignOrSemicolon,
    BraceOrCommaOrParenthesis,
    BraceOrParenthesis,
    ColonOrComma,
    CommaOrDot,
    CommaOrGt,
    DotOrSemicolon,
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
