use crate::ErrorToken;
use sodigy_name_analysis::NameKind;
use sodigy_string::InternedString;

mod render;

#[derive(Clone, Debug)]
pub enum ErrorKind {
    InvalidNumberLiteral,
    InvalidStringLiteralPrefix,
    InvalidCharacterInIdentifier(char),
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
    InclusiveRangeWithNoEnd,
    AstPatternTypeError,  // TODO: more context?
    InvalidFnType,
    EmptyMatchStatement,
    RedundantDecorator(InternedString),

    // TODO: suggest similar names
    // TODO: tell what it's trying to decorator
    InvalidDecorator(InternedString),

    // Syntax errors in patterns
    CannotBindNameToAnotherName(InternedString),
    CannotAnnotateType,
    RedundantNameBinding(InternedString, InternedString),

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

    // TODO: We need more helpful error variants
    //       e.g. if we know the types, we can guess which argument is missing, or surplus, or in different order
    MissingArgument {
        expected: usize,
        got: usize,
    },
    UnexpectedArgument {
        expected: usize,
        got: usize,
    },

    StructFieldRepeated(InternedString),
    MissingStructField(InternedString),

    // TODO: suggest similar name
    InvalidStructField(InternedString),

    // --- warnings from here ---
    UnusedName {
        name: InternedString,
        kind: NameKind,
    },
}
