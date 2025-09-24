use sodigy_keyword::Keyword;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod error;

pub use error::ErrorToken;

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(InternedString),
    Number(InternedNumber),
    String(InternedString),
    Char(char),
    FieldModifier(InternedString),
    Decorator(InternedString),
    DocComment(InternedString),
    Punct(Punct),

    // It'll be later processed to `Group` by `group_tokens`
    GroupDelim {
        // Only opening delims have this value. Closing delims don't need this.
        delim: Option<Delim>,

        // It's used by `group_tokens` to match delims.
        id: Span,
    },
    Group {
        delim: Delim,
        tokens: Vec<Token>,
    },
}

impl TokenKind {
    pub fn matches(&self, other: &TokenKind) -> bool {
        // DO NOT USE a wildcard pattern. At least one of `self` or `other` must be matched.
        // Otherwise, it'd be error prone when a new TokenKind is added.
        match (self, other) {
            (TokenKind::Keyword(a), TokenKind::Keyword(b)) => a == b,
            (TokenKind::Keyword(_), _) => false,
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::Identifier(_), _) => false,
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (TokenKind::Number(_), _) => false,
            (TokenKind::Decorator(_), TokenKind::Decorator(_)) => true,
            (TokenKind::Decorator(_), _) => false,
            (TokenKind::DocComment(_), TokenKind::DocComment(_)) => true,
            (TokenKind::DocComment(_), _) => false,
            (TokenKind::Punct(a), TokenKind::Punct(b)) => a == b,
            (TokenKind::Punct(_), _) => false,
            (TokenKind::Group { delim: a, .. }, TokenKind::Group { delim: b, .. }) => a == b,
            (TokenKind::Group { .. }, _) => false,
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delim {
    Parenthesis,
    Bracket,
    Brace,
    Lambda,  // \{}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Punct {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Colon,
    Semicolon,
    Lt,
    Eq,
    Gt,
    Comma,
    Dot,
}

impl From<u8> for Punct {
    fn from(b: u8) -> Punct {
        match b {
            b'%' => Punct::Rem,
            b'*' => Punct::Mul,
            b'+' => Punct::Add,
            b',' => Punct::Comma,
            b'-' => Punct::Sub,
            b'/' => Punct::Div,
            b':' => Punct::Colon,
            b';' => Punct::Semicolon,
            b'<' => Punct::Lt,
            b'=' => Punct::Eq,
            b'>' => Punct::Gt,
            b'.' => Punct::Dot,
            _ => panic!("TODO: {:?}", b as char),
        }
    }
}
