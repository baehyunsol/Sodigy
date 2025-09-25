use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::ErrorToken;

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,

    // Some errors have multiple spans (e.g. name collision)
    pub extra_span: Option<Span>,
    pub extra_message: Option<String>,
}

impl Default for Error {
    fn default() -> Error {
        Error {
            // please don't use this value
            kind: ErrorKind::InvalidUtf8,
            span: Span::None,

            // default is for these fields
            extra_span: None,
            extra_message: None,
        }
    }
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
    NameCollision {
        name: InternedString,

        // TODO
        // context: NameCollisionContext,
    },
}
