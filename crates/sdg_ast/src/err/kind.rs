use super::ExpectedToken;
use crate::session::{InternedString, LocalParseSession};
use crate::token::TokenKind;
use crate::utils::bytes_to_string;
use sdg_fs::FileError;

#[derive(PartialEq)]
pub enum ParseErrorKind {
    // only for lexer
    UnexpectedChar(char),
    UnexpectedEof,
    InvalidUTF8(Vec<u8>),

    FileError(FileError),

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
    MultipleDefParam(InternedString, ParamType),
}

#[derive(Copy, Clone, PartialEq)]
pub enum ParamType {
    FuncParam,
    LambdaParam,
    BlockDef,
}

impl ParseErrorKind {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ParseErrorKind::UnexpectedChar(c) => format!(
                "unexpected character: {c:?}"
            ),
            ParseErrorKind::UnexpectedEof => format!("unexpected end of file"),
            ParseErrorKind::UnexpectedEoe(expected) => format!(
                "{}, found nothing",
                expected.render_err(session)
            ),
            ParseErrorKind::UnexpectedToken { expected, got } => format!(
                "{}, found {}",
                expected.render_err(session),
                got.render_err(session),
            ),
            ParseErrorKind::InvalidUTF8(chars) => format!(
                "{chars:?} is not a valid utf-8"
            ),
            ParseErrorKind::UntypedArg(arg, func) => format!(
                "expected a type annotation, found nothing\nParameter `{}` of function `{}` has no type annotation.\nOnly lambda functions are allowed to omit type annotations.",
                bytes_to_string(&session.unintern_string(*arg)),
                bytes_to_string(&session.unintern_string(*func)),
            ),
            ParseErrorKind::MultipleDefParam(name, param_type) => format!(
                "identifier `{}` is bound more than once in {}",
                bytes_to_string(&session.unintern_string(*name)),
                param_type.render_err(),
            ),
            ParseErrorKind::FileError(e) => e.render_err(),
        }
    }
}

impl ParamType {
    pub fn render_err(&self) -> String {
        match self {
            ParamType::FuncParam => "a function parameter list",
            ParamType::LambdaParam => "a lambda parameter list",
            ParamType::BlockDef => "a block expression",
        }.to_string()
    }
}
