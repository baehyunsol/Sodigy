use crate::session::LocalParseSession;
use crate::token::TokenKind;

#[derive(PartialEq)]
pub enum ParseErrorKind {
    UnexpectedChar(char),

    // only for lexers
    UnexpectedEof,

    // expected an expression, but got nothing
    UnexpectedEoe,

    UnexpectedToken { expected: Vec<TokenKind>, got: TokenKind }
}

impl ParseErrorKind {

    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ParseErrorKind::UnexpectedChar(c) => format!("Unexpected character `{c}` is found while parsing!"),
            ParseErrorKind::UnexpectedEof => format!("Unexpected EOF while parsing!"),
            ParseErrorKind::UnexpectedEoe => format!("Expected an expression, but got nothing!"),
            ParseErrorKind::UnexpectedToken { expected, got } => if expected.len() == 0 {
                format!("Unexpected Token: `{}`", got.render_err(session))
            } else {
                format!(
                    "Expected `{}`, but got `{}`",
                    expected.iter().map(
                        |token| token.render_err(session)
                    ).collect::<Vec<String>>().join(", "),
                    got.render_err(session)
                )
            }
        }
    }

}