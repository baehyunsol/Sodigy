use super::ExpectedToken;
use crate::session::LocalParseSession;
use crate::token::TokenKind;

#[derive(PartialEq)]
pub enum ParseErrorKind {
    // only for lexer
    UnexpectedChar(char),
    UnexpectedEof,
    InvalidUTF8(Vec<u8>),

    // expected something, but got nothing
    UnexpectedEoe(ExpectedToken),

    UnexpectedToken {
        expected: ExpectedToken,
        got: TokenKind,
    },
}

impl ParseErrorKind {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ParseErrorKind::UnexpectedChar(c) => format!(
                "Unexpected character `{c}` is found while parsing!"
            ),
            ParseErrorKind::UnexpectedEof => format!("Unexpected EOF while parsing!"),
            ParseErrorKind::UnexpectedEoe(expected) => format!(
                "{} but got nothing!",
                expected.render_err(session)
            ),
            ParseErrorKind::UnexpectedToken { expected, got } => format!(
                "{} but got {}",
                expected.render_err(session),
                got.render_err(session),
            ),
            ParseErrorKind::InvalidUTF8(chars) => format!(
                "Invalid UTF-8 bytes are found: {chars:?}"
            ),
        }
    }
}
