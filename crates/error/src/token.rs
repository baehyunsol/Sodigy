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
    Ident,
    Generic,
    Number,
    String,
    TypeAnnot,
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

impl ErrorToken {
    pub fn unwrap_punct(&self) -> Punct {
        match self {
            ErrorToken::Punct(p) => *p,
            _ => panic!(),
        }
    }
}

impl From<&TokenKind> for ErrorToken {
    fn from(t: &TokenKind) -> ErrorToken {
        match t {
            TokenKind::Keyword(k) => ErrorToken::Keyword(*k),
            TokenKind::Ident(_) => ErrorToken::Ident,
            TokenKind::Number(_) => ErrorToken::Number,
            TokenKind::String { .. } => ErrorToken::String,
            TokenKind::Punct(p) => ErrorToken::Punct(*p),
            TokenKind::Group { delim, .. } => ErrorToken::Group(*delim),
            _ => panic!("TODO: {t:?}"),
        }
    }
}
