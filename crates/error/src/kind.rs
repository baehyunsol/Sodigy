use crate::ErrorToken;
use sodigy_string::InternedString;

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
    DocCommentNotAllowed,
    DecoratorNotAllowed,
    BlockWithoutValue,
    StructWithoutField,
    EmptyCurlyBraceBlock,
    PositionalArgAfterKeywordArg,
    NonDefaultValueAfterDefaultValue,
    CannotDeclareInlineModule,
    NameCollision {
        name: InternedString,

        // TODO
        // context: NameCollisionContext,
    },

    // TODO: suggest similar names
    UndefinedName(InternedString),

    KeywordArgumentRepeated(InternedString),
    KeywordArgumentNotAllowed,

    // TODO: suggest similar names
    InvalidKeywordArgument(InternedString),

    StructFieldRepeated(InternedString),
    MissingStructField(InternedString),

    // TODO: suggest similar name
    InvalidStructField(InternedString),

    // --- warnings from here ---
    UnusedName(InternedString),
}
