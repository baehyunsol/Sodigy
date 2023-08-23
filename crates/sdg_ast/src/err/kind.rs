use super::ExpectedToken;
use crate::ast::NameOrigin;
use crate::pattern::PatternErrorKind;
use crate::session::{InternedString, LocalParseSession};
use crate::token::TokenKind;
use sdg_fs::FileError;

#[derive(Clone, PartialEq)]
pub enum ParseErrorKind {
    /// only for lexer
    UnexpectedChar(char),
    UnexpectedEof,
    InvalidUTF8(Vec<u8>),

    FileError(FileError),

    /// 'abc' and '' are invalid, it must be a single character\
    /// `usize` is for length of the given literal
    InvalidCharLiteral(usize),

    /// expected something, but got nothing
    UnexpectedEoe(ExpectedToken),

    UnexpectedToken {
        expected: ExpectedToken,
        got: TokenKind,
    },

    /// A definition of a lambda may omit type notations, but `def` may not
    UntypedArg(InternedString, InternedString),

    /// `def foo(x: Int, x: Int)`\
    /// `\{x: Int, x: Int, x + x}`\
    /// `{x = 3; x = 4; x + x}`\
    MultipleDefParam(InternedString, ParamType),

    LambdaHashCollision,

    PatternFromArg(InternedString, NameOrigin),

    InvalidPattern(PatternErrorKind),
}

#[derive(Copy, Clone, PartialEq)]
pub enum ParamType {
    FuncParam,
    LambdaParam,
    FuncGeneric,
    FuncGenericAndParam,
    BlockDef,
    PatternNameBinding,
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
            ParseErrorKind::InvalidCharLiteral(len) => if *len == 0 {
                "empty character literal".to_string()
            } else {
                "character literal may only contain one codepoint".to_string()
            },
            ParseErrorKind::UntypedArg(arg, func) => format!(
                "expected a type annotation, found nothing\nParameter `{}` of function `{}` has no type annotation.\nOnly lambda functions are allowed to omit type annotations.",
                arg.to_string(session),
                func.to_string(session),
            ),
            ParseErrorKind::MultipleDefParam(name, param_type) => format!(
                "identifier `{}` is bound more than once in {}",
                name.to_string(session),
                param_type.render_err(),
            ),
            ParseErrorKind::LambdaHashCollision => String::from(
                "hash collision in lambda functions"
            ),
            ParseErrorKind::PatternFromArg(name, _) => format!(
                "cannot use `{}` inside a pattern",
                name.to_string(session),
            ),
            ParseErrorKind::FileError(e) => e.render_err(),
            ParseErrorKind::InvalidPattern(p) => p.render_err(session),
        }
    }
}

impl ParamType {
    pub fn render_err(&self) -> String {
        match self {
            ParamType::FuncParam => "a function parameter list",
            ParamType::LambdaParam => "a lambda parameter list",
            ParamType::BlockDef => "a block expression",
            ParamType::FuncGeneric => "a function generic parameter list",
            ParamType::PatternNameBinding => "a pattern name binding list",
            ParamType::FuncGenericAndParam =>
                "a function generic parameter list and a parameter list",
        }.to_string()
    }
}
