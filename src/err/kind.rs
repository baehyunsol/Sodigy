use super::ExpectedToken;
use crate::session::{InternedString, LocalParseSession};
use crate::token::TokenKind;
use crate::utils::bytes_to_string;

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

    // A definition of a lambda may omit type notations, but `def` may not
    UntypedArg(InternedString, InternedString),

    // def foo(x: Int, x: Int)
    // \{x: Int, x: Int, x + x}
    // {x = 3; x = 4; x + x}
    MultipleDefParam(InternedString),
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
            ParseErrorKind::UntypedArg(arg, func) => format!(
                "Argument `{}` of function `{}` has no type annotation!\nIn lambda functions, you may omit type annotations, but not with `def`s.",
                bytes_to_string(&session.unintern_string(*arg)),
                bytes_to_string(&session.unintern_string(*func)),
            ),
            ParseErrorKind::MultipleDefParam(name) => format!(
                "Parameter `{}` is defined more than once!",
                bytes_to_string(&session.unintern_string(*name)),
            ),
        }
    }
}
