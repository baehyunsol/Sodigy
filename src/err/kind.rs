use crate::session::LocalParseSession;
use crate::token::TokenKind;

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedChar(char),

    // only for lexers
    UnexpectedEof,

    // expected an expression, but got nothing
    UnexpectedEoe,

    UnexpectedToken(TokenKind)
}

impl ParseErrorKind {

    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ParseErrorKind::UnexpectedChar(c) => format!("Unexpected character `{c}` is found while parsing!"),
            ParseErrorKind::UnexpectedEof => format!("Unexpected EOF while parsing!"),
            ParseErrorKind::UnexpectedEoe => format!("Expected an expression, but got nothing!"),
            ParseErrorKind::UnexpectedToken(t) => format!("Unexpected token `{}`", t.render_err(session))
        }
    }

}