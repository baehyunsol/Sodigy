use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod delim;
mod keyword;
mod op;
mod punct;

pub use delim::Delim;
pub use keyword::Keyword;
pub use op::{InfixOp, PostfixOp, PrefixOp};
pub use punct::Punct;

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    // A token that can be a beginning of a pattern.
    pub fn pattern_begin(&self) -> bool {
        self.kind.pattern_begin()
    }

    // A token that can be a beginning of an expr.
    pub fn expr_begin(&self) -> bool {
        self.kind.expr_begin()
    }
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(InternedString),
    Number(InternedNumber),
    String {
        binary: bool,
        raw: bool,
        s: InternedString,
    },
    Char {
        binary: bool,
        ch: u32,
    },
    FieldModifier(InternedString),
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
            (TokenKind::DocComment(_), TokenKind::DocComment(_)) => true,
            (TokenKind::DocComment(_), _) => false,
            (TokenKind::Punct(a), TokenKind::Punct(b)) => a == b,
            (TokenKind::Punct(_), _) => false,
            (TokenKind::Group { delim: a, .. }, TokenKind::Group { delim: b, .. }) => a == b,
            (TokenKind::Group { .. }, _) => false,
            _ => todo!(),
        }
    }

    // A token that can be a beginning of a pattern.
    pub fn pattern_begin(&self) -> bool {
        match self {
            TokenKind::Identifier(_) |
            TokenKind::Number(_) |
            TokenKind::String { .. } |
            TokenKind::Char { .. } => true,

            TokenKind::Keyword(_) |
            TokenKind::FieldModifier(_) |
            TokenKind::DocComment(_) |
            TokenKind::GroupDelim { delim: None, .. } => false,

            TokenKind::Punct(p) => match p {
                Punct::Dollar |
                Punct::DotDot |
                Punct::DotDotEq => true,
                _ => false,
            },

            TokenKind::GroupDelim { delim: Some(delim), .. } |
            TokenKind::Group { delim, .. } => match delim {
                Delim::Parenthesis |
                Delim::Bracket => true,
                Delim::Brace |
                Delim::Lambda => false,
            },
        }
    }

    pub fn expr_begin(&self) -> bool {
        match self {
            TokenKind::Identifier(_) |
            TokenKind::Number(_) |
            TokenKind::String { .. } |
            TokenKind::Char { .. } => true,

            TokenKind::FieldModifier(_) |
            TokenKind::DocComment(_) |
            TokenKind::GroupDelim { delim: None, .. } => false,

            TokenKind::Punct(p) => PrefixOp::try_from(*p).is_ok(),
            TokenKind::Keyword(k) => match k {
                Keyword::Let |
                Keyword::Fn |
                Keyword::Struct |
                Keyword::Enum |
                Keyword::Assert |
                Keyword::Type |
                Keyword::Module |
                Keyword::Use |
                Keyword::Pub |
                Keyword::Else => false,

                Keyword::If |
                Keyword::Match => true,
            },

            TokenKind::GroupDelim { delim: Some(delim), .. } |
            TokenKind::Group { delim, .. } => true,
        }
    }
}
