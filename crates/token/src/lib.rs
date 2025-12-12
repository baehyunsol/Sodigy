use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod delim;
mod endec;
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

    pub fn offset_span(&mut self, offset: usize) {
        match &mut self.kind {
            TokenKind::FormattedString { elements, .. } => {
                for element in elements.iter_mut() {
                    match element {
                        TokensOrString::Tokens { tokens, span } => {
                            for token in tokens.iter_mut() {
                                token.offset_span(offset);
                            }

                            span.offset(offset);
                        },
                        _ => {},
                    }
                }
            },
            TokenKind::Group { tokens, .. } => {
                for token in tokens.iter_mut() {
                    token.offset_span(offset);
                }
            },
            _ => {},
        }

        self.span.offset(offset);
    }
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    Keyword(Keyword),
    Ident(InternedString),
    Number(InternedNumber),
    String {
        binary: bool,
        raw: bool,
        regex: bool,  // it implies `raw = true` and `binary = false`
        s: InternedString,
    },
    // `'a'`
    Char(u32),
    // `b'a'` or `200b`
    // `181b` is a valid byte, but `b'µ'` is not.
    // The `b'_'` syntax is only valid for ascii characters.
    Byte(u8),
    FormattedString {
        raw: bool,

        // It filters out empty strings.
        // For example, `f""`'s `elements` is `vec![]`.
        elements: Vec<TokensOrString>,
    },
    FieldModifier(InternedString),
    DocComment {
        // `//!`
        top_level: bool,
        doc: InternedString,
    },
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
            (TokenKind::Ident(_), TokenKind::Ident(_)) => true,
            (TokenKind::Ident(_), _) => false,
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (TokenKind::Number(_), _) => false,
            (TokenKind::String { .. }, TokenKind::String { .. }) => true,
            (TokenKind::String { .. }, _) => false,
            (TokenKind::Char(_), TokenKind::Char(_)) => true,
            (TokenKind::Char(_), _) => false,
            (TokenKind::Byte(_), TokenKind::Byte(_)) => true,
            (TokenKind::Byte(_), _) => false,
            (TokenKind::FormattedString { .. }, TokenKind::FormattedString { .. }) => true,
            (TokenKind::FormattedString { .. }, _) => false,
            (TokenKind::FieldModifier(_), TokenKind::FieldModifier(_)) => true,
            (TokenKind::FieldModifier(_), _) => false,
            (TokenKind::DocComment { top_level: a, .. }, TokenKind::DocComment { top_level: b, .. }) => a == b,
            (TokenKind::DocComment { .. }, _) => false,
            (TokenKind::Punct(a), TokenKind::Punct(b)) => a == b,
            (TokenKind::Punct(_), _) => false,
            (TokenKind::GroupDelim { .. }, _) | (_, TokenKind::GroupDelim { .. }) => unreachable!(),
            (TokenKind::Group { delim: a, .. }, TokenKind::Group { delim: b, .. }) => a == b,
            (TokenKind::Group { .. }, _) => false,
        }
    }

    // A token that can be a beginning of a pattern.
    pub fn pattern_begin(&self) -> bool {
        match self {
            TokenKind::Keyword(k) => match k {
                _ => todo!(),
            },
            TokenKind::Punct(p) => match p {
                Punct::Sub |
                Punct::Dollar |
                Punct::DotDot |
                Punct::DotDotEq => true,
                _ => false,
            },
            TokenKind::Group { delim, .. } => match delim {
                _ => todo!(),
            },
            TokenKind::Ident(_) |
            TokenKind::Number(_) |
            TokenKind::String { .. } |
            TokenKind::Char(_) |
            TokenKind::Byte(_) => true,
            TokenKind::FormattedString { .. } |
            TokenKind::FieldModifier(_) |
            TokenKind::DocComment { .. } |
            TokenKind::GroupDelim { .. } => false,
        }
    }

    pub fn expr_begin(&self) -> bool {
        match self {
            TokenKind::Ident(_) |
            TokenKind::Number(_) |
            TokenKind::String { .. } |
            TokenKind::Char(_) |
            TokenKind::Byte(_) |
            TokenKind::FormattedString { .. } => true,

            TokenKind::FieldModifier(_) |
            TokenKind::DocComment { .. } |
            TokenKind::GroupDelim { delim: None, .. } => false,

            TokenKind::Punct(p) => PrefixOp::try_from(*p).is_ok(),
            TokenKind::Keyword(k) => match k {
                Keyword::As |
                Keyword::Assert |
                Keyword::Else |
                Keyword::Enum |
                Keyword::Fn |
                Keyword::Let |
                Keyword::Mod |
                Keyword::Pub |
                Keyword::Struct |
                Keyword::Type |
                Keyword::Use => false,

                Keyword::If |
                Keyword::Match => true,
            },

            TokenKind::GroupDelim { delim: Some(_), .. } |
            TokenKind::Group { .. } => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TokensOrString {
    Tokens {
        tokens: Vec<Token>,
        span: Span,
    },
    String(InternedString),
}
