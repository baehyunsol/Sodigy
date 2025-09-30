use crate::{ErrorKind, WarningKind};

// TODO: maybe more levels?
pub enum ErrorLevel {
    Error,
    Warning,
}

impl ErrorLevel {
    pub fn from_error_kind(k: &ErrorKind) -> ErrorLevel {
        match k {
            ErrorKind::InvalidNumberLiteral |
            ErrorKind::InvalidStringLiteralPrefix |
            ErrorKind::WrongNumberOfQuotesInRawStringLiteral |
            ErrorKind::UnterminatedStringLiteral |
            ErrorKind::InvalidCharLiteral |
            ErrorKind::InvalidCharLiteralPrefix |
            ErrorKind::UnterminatedCharLiteral |
            ErrorKind::InvalidEscape |
            ErrorKind::EmptyCharLiteral |
            ErrorKind::UnterminatedBlockComment |
            ErrorKind::InvalidUtf8 |
            ErrorKind::TooManyQuotes |
            ErrorKind::UnclosedDelimiter(_) |
            ErrorKind::UnexpectedToken { .. } |
            ErrorKind::UnexpectedEof { .. } |
            ErrorKind::UnexpectedEog { .. } |
            ErrorKind::DocCommentForNothing |
            ErrorKind::DocCommentNotAllowed |
            ErrorKind::DecoratorNotAllowed |
            ErrorKind::BlockWithoutValue |
            ErrorKind::StructWithoutField |
            ErrorKind::EmptyCurlyBraceBlock |
            ErrorKind::PositionalArgAfterKeywordArg |
            ErrorKind::NonDefaultValueAfterDefaultValue |
            ErrorKind::CannotDeclareInlineModule |
            ErrorKind::NameCollision { .. } |
            ErrorKind::UndefinedName(_) |
            ErrorKind::KeywordArgumentRepeated(_) |
            ErrorKind::KeywordArgumentNotAllowed |
            ErrorKind::InvalidKeywordArgument(_) |
            ErrorKind::StructFieldRepeated(_) |
            ErrorKind::MissingStructField(_) |
            ErrorKind::InvalidStructField(_) => ErrorLevel::Error,
            WarningKind::UnusedName(_) => ErrorLevel::Warning,
        }
    }
}
