use crate::{Delim, formatted_str::FormattedStringElement, punct::Punct};
use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_keyword::Keyword;
use sodigy_lex::QuoteKind;
use sodigy_span::SpanRange;

mod fmt;

#[derive(Clone, Debug)]
pub struct TokenTree {
    pub kind: TokenTreeKind,
    pub span: SpanRange,
}

impl TokenTree {
    pub fn new_ident(ident: InternedString, span: SpanRange) -> Self {
        TokenTree {
            kind: TokenTreeKind::Identifier(ident),
            span,
        }
    }
    pub fn new_keyword(keyword: Keyword, span: SpanRange) -> Self {
        TokenTree {
            kind: TokenTreeKind::Keyword(keyword),
            span,
        }
    }

    pub fn new_doc_comment(doc_comment: InternedString, span: SpanRange) -> Self {
        TokenTree {
            kind: TokenTreeKind::DocComment(doc_comment),
            span,
        }
    }

    pub fn new_punct(punct: Punct, span: SpanRange) -> Self {
        TokenTree {
            kind: TokenTreeKind::Punct(punct),
            span,
        }
    }

    pub fn new_group(delim: Delim, span: SpanRange) -> Self {
        TokenTree {
            kind: TokenTreeKind::Group {
                delim,
                tokens: vec![],
                prefix: b'\0',
            },
            span,
        }
    }

    pub fn remove_prefix(&mut self) {
        self.kind.remove_prefix();
    }
}

#[derive(Clone, Debug)]
pub enum TokenTreeKind {
    Identifier(InternedString),
    Keyword(Keyword),
    Number(InternedNumeric),

    Punct(Punct),
    Group {
        delim: Delim,
        tokens: Vec<TokenTree>,

        // for now, it's either b'\0' or b'\\'
        // b'\0' for no prefix
        prefix: u8,
    },
    String {
        kind: QuoteKind,
        content: InternedString,
        is_binary: bool,  // prefixed with `b`
    },

    FormattedString(Vec<FormattedStringElement>), // prefixed with `f`
    DocComment(InternedString),
}

impl TokenTreeKind {
    pub fn remove_prefix(&mut self) {
        match self {
            TokenTreeKind::Group { prefix, .. } => {
                *prefix = b'\0';
            },
            _ => unreachable!(),
        }
    }
}

impl PartialEq for TokenTreeKind {
    fn eq(&self, other: &TokenTreeKind) -> bool {
        // Don't use (self, other), use nested match statements
        // so that the compiler warns you when new TokenTreeKind variants are added
        match self {
            TokenTreeKind::Identifier(id1) => match other {
                TokenTreeKind::Identifier(id2) => id1 == id2,
                _ => false,
            },
            TokenTreeKind::Keyword(k1) => match other {
                TokenTreeKind::Keyword(k2) => k1 == k2,
                _ => false,
            },
            TokenTreeKind::Punct(p1) => match other {
                TokenTreeKind::Punct(p2) => p1 == p2,
                _ => false,
            },
            TokenTreeKind::Number(n1) => match other {
                TokenTreeKind::Number(n2) => n1 == n2,
                _ => false,
            },
            TokenTreeKind::DocComment(d1) => match other {
                TokenTreeKind::DocComment(d2) => d1 == d2,
                _ => false,
            },
            TokenTreeKind::String { content: s1, .. } => match other {
                TokenTreeKind::String { content: s2, .. } => s1 == s2,
                _ => false,
            },

            // are you sure?
            TokenTreeKind::Group { .. }
            | TokenTreeKind::FormattedString(_) => false,
        }
    }
}
