use super::{Delimiter, Keyword, OpToken, Token};
use crate::session::{InternedString, LocalParseSession};
use crate::utils::bytes_to_string;
use hmath::Ratio;

#[derive(Clone, PartialEq)]
pub enum TokenKind {
    Number(Ratio),
    String(InternedString),

    // It doesn't care how the inside looks like. It only guarantees that the opening and the closing are properly matched.
    List(Delimiter, Vec<Box<Token>>),
    Identifier(InternedString),

    // True, False, None
    Keyword(Keyword),

    Operator(OpToken),
}

impl TokenKind {
    pub fn is_identifier(&self) -> bool {
        if let TokenKind::Identifier(_) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap_identifier(&self) -> InternedString {
        if let TokenKind::Identifier(s) = self {
            *s
        } else {
            panic!(
                "Internal Compiler Error 0E82A87: {}",
                self.render_err(&LocalParseSession::dummy())
            )
        }
    }

    pub fn dummy_identifier() -> Self {
        TokenKind::Identifier(InternedString::dummy())
    }

    // preview of this token_kind for error messages
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            TokenKind::Number(_) => "Number".to_string(),
            TokenKind::String(_) => "String Literal".to_string(),
            TokenKind::List(delim, _) => match delim {
                Delimiter::Parenthesis => "(...)",
                Delimiter::Brace => "{...}",
                Delimiter::Bracket => "[...]",
            }
            .to_string(),
            TokenKind::Identifier(string) => {
                if string.is_dummy() || session.is_dummy {
                    "Identifier".to_string()
                } else {
                    format!(
                        "Identifier: `{}`",
                        bytes_to_string(&session.unintern_string(*string)),
                    )
                }
            }
            TokenKind::Keyword(k) => format!("Keyword: `{}`", k.render_err()),
            TokenKind::Operator(op) => format!("Special Character: `{}`", op.render_err()),
        }
    }

    #[cfg(test)]
    pub fn is_same_type(&self, other: &TokenKind) -> bool {
        match (self, other) {
            (TokenKind::Number(m), TokenKind::Number(n)) => m == n,
            (TokenKind::Keyword(m), TokenKind::Keyword(n)) => m == n,
            (TokenKind::Operator(m), TokenKind::Operator(n)) => m == n,

            // test runners do not care about the elements:
            // because the error variants do not care about the elements!
            (TokenKind::List(m, _), TokenKind::List(n, _)) => m == n,

            // test runners cannot generate the same string: they cannot access the ParseSession
            (TokenKind::String(_), TokenKind::String(_)) => true,

            // test runners cannot generate the same identifier: they cannot access the ParseSession
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,

            _ => false,
        }
    }
}
