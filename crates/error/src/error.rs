use sodigy_span::Span;
use sodigy_token::ErrorToken;

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    InvalidNumberLiteral,
    InvalidStringLiteralPrefix,
    WrongNumberOfQuotesInRawStringLiteral,
    UnterminatedStringLiteral,
    InvalidCharLiteral,
    InvalidCharLiteralPrefix,
    UnterminatedCharLiteral,
    InvalidEscape,
    EmptyCharLiteral,
    UnterminatedBlockComment,
    InvalidUtf8,

    // You can use up to 127 quotes for opening or 254 quotes (open 127 + close 127) consecutively.
    TooManyQuotes,
    UnclosedDelimiter(u8),
    UnexpectedToken {
        expected: ErrorToken,
        got: ErrorToken,
    },
    UnexpectedEof {
        expected: ErrorToken,
    },
    // It's like Eof, but an end of a group (parenthesis, braces or brackets).
    UnexpectedEog {
        expected: ErrorToken,
    },
    DocCommentForNothing,
    BlockWithoutValue,
    StructWithoutField,
    EmptyCurlyBraceBlock,
    PositionalArgAfterKeywordArg,
    NonDefaultValueAfterDefaultValue,
}
