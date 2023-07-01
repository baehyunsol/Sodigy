use super::{Delimiter, Keyword, OpToken, Token};
use crate::session::{InternedString, LocalParseSession};
use hmath::Ratio;

#[derive(Clone, PartialEq)]
pub enum TokenKind {
    Number(Ratio),
    String(InternedString),

    // It doesn't care how the inside looks like. It only guarantees that the opening and closing are properly matched.
    List(Delimiter, Vec<Box<Token>>),
    Identifier(InternedString),

    // True, False, None
    Keyword(Keyword),

    Operator(OpToken)
}

impl TokenKind {

    pub fn is_identifier(&self) -> bool {

        if let TokenKind::Identifier(_) = self {
            true
        }

        else {
            false
        }

    }

    pub fn unwrap_identifier(&self) -> InternedString {

        if let TokenKind::Identifier(s) = self {
            *s
        }

        else {
            panic!("Internal Compiler Error 0E82A87: {}", self.render_err(&LocalParseSession::dummy()))
        }

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
            }.to_string(),
            TokenKind::Identifier(string) => if string.is_dummy() || session.is_dummy {
                "Identifier".to_string()
            } else {
                format!(
                    "Identifier: `{}`",
                    String::from_utf8_lossy(&session.unintern_string(*string).unwrap_or(vec![b'?'; 3])).to_string()
                )
            },
            TokenKind::Keyword(k) => format!("Keyword: `{}`", k.render_err()),
            TokenKind::Operator(op) => format!("Special Character: `{}`", op.render_err())
        }
    }

}