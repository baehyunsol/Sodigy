use crate::{CommentKind, QuoteKind};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

mod fmt;

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SpanRange,
}

impl Token {
    #[inline]
    pub fn is_whitespace(&self) -> bool {
        self.kind.is_whitespace()
    }
}

#[derive(Clone)]
pub enum TokenKind {
    // span includes '#'s and '\n'
    Comment { kind: CommentKind, content: String },

    // span includes quotes
    String { kind: QuoteKind, content: String },

    Identifier(InternedString),

    // it doesn't include '_'s
    Number(Vec<u8>),

    // it doesn't care whether it's ' ', '\t' or '\n'
    // whitespace tokens don't have spans
    Whitespace,
    Punct(u8),

    // '(', ')', '[', ']', '{', '}'
    Grouper(u8),
}

impl TokenKind {
    pub fn try_lex_punct(c: u8) -> Result<Self, ()> {
        if c == b'(' || c == b')'
        || c == b'{' || c == b'}'
        || c == b'[' || c == b']' {
            Ok(TokenKind::Grouper(c))
        }

        else if is_valid_punct(c) {
            Ok(TokenKind::Punct(c))
        }

        else {
            Err(())
        }
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Whitespace)
    }
}

fn is_valid_punct(c: u8) -> bool {
    match c {
        33 | 36..=38 | 42..=47 | 58..=64
        | 92 | 94 | 96 | 124 | 126 => true,
        _ => false,
    }
}
