use crate::{ErrorKind, WarningKind};
use sodigy_span::Color;

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
            ErrorKind::InvalidCharacterInIdentifier(_) |
            ErrorKind::WrongNumberOfQuotesInRawStringLiteral |
            ErrorKind::UnterminatedStringLiteral |
            ErrorKind::NotAllowedCharInFString(_) |
            ErrorKind::InvalidCharLiteral |
            ErrorKind::InvalidCharLiteralPrefix |
            ErrorKind::UnterminatedCharLiteral |
            ErrorKind::InvalidByteLiteral |
            ErrorKind::InvalidEscape |
            ErrorKind::EmptyCharLiteral |
            ErrorKind::UnterminatedBlockComment |
            ErrorKind::InvalidUtf8 |
            ErrorKind::InvalidUnicodeCharacter |
            ErrorKind::InvalidUnicodeEscape |
            ErrorKind::UnmatchedGroup { .. } |
            ErrorKind::TooManyQuotes |
            ErrorKind::UnclosedDelimiter(_) |
            ErrorKind::UnexpectedToken { .. } |
            ErrorKind::UnexpectedEof { .. } |
            ErrorKind::UnexpectedEog { .. } |
            ErrorKind::MissingDocComment |
            ErrorKind::DocCommentNotAllowed |
            ErrorKind::ModuleDocCommentNotAtTop |
            ErrorKind::MissingDecorator(_) |
            ErrorKind::DecoratorNotAllowed |
            ErrorKind::UnexpectedDecorator(_) |
            ErrorKind::ModuleDecoratorNotAtTop |
            ErrorKind::MissingVisibility |
            ErrorKind::CannotBePublic |
            ErrorKind::FunctionWithoutBody |
            ErrorKind::BlockWithoutValue |
            ErrorKind::StructWithoutField |
            ErrorKind::EmptyCurlyBraceBlock |
            ErrorKind::PositionalArgAfterKeywordArg |
            ErrorKind::NonDefaultValueAfterDefaultValue |
            ErrorKind::CannotDeclareInlineModule |
            ErrorKind::InclusiveRangeWithNoEnd |
            ErrorKind::AstPatternTypeError |
            ErrorKind::DifferentNameBindingsInOrPattern |
            ErrorKind::InvalidFnType |
            ErrorKind::EmptyMatchStatement |
            ErrorKind::RedundantDecorator(_) |
            ErrorKind::InvalidDecorator(_) |
            ErrorKind::MissingDecoratorArgument { .. } |
            ErrorKind::UnexpectedDecoratorArgument { .. } |
            ErrorKind::CannotBindNameToAnotherName(_) |
            ErrorKind::CannotAnnotateType |

            // Rust treats it as a warning, but Sodigy treats it as an error
            // because it messes up with some analysis
            ErrorKind::RedundantNameBinding(_, _) |

            ErrorKind::NameCollision { .. } |
            ErrorKind::CyclicLet { .. } |
            ErrorKind::CyclicAlias { .. } |
            ErrorKind::UndefinedName(_) |
            ErrorKind::KeywordArgumentRepeated(_) |
            ErrorKind::KeywordArgumentNotAllowed |
            ErrorKind::AliasResolveRecursionLimitReached |
            ErrorKind::MissingTypeParameter { .. } |
            ErrorKind::UnexpectedTypeParameter { .. } |
            ErrorKind::MissingKeywordArgument(_) |
            ErrorKind::InvalidKeywordArgument(_) |
            ErrorKind::MissingFunctionParameter { .. } |
            ErrorKind::UnexpectedFunctionParameter { .. } |
            ErrorKind::StructFieldRepeated(_) |
            ErrorKind::MissingStructField(_) |
            ErrorKind::InvalidStructField(_) |
            ErrorKind::DependentTypeNotAllowed |
            ErrorKind::UnexpectedType { .. } |
            ErrorKind::CannotInferType { .. } |
            ErrorKind::PartiallyInferedType { .. } |
            ErrorKind::CannotInferGenericType { .. } |
            ErrorKind::PartiallyInferedGenericType { .. } |
            ErrorKind::CannotApplyInfixOp { .. } |
            ErrorKind::CannotSpecializePolyGeneric { .. } |
            ErrorKind::MultipleModuleFiles { .. } |
            ErrorKind::ModuleFileNotFound { .. } |
            ErrorKind::LibFileNotFound |
            ErrorKind::Todo { .. } => ErrorLevel::Error,
            WarningKind::UnusedNames { .. } => ErrorLevel::Warning,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ErrorLevel::Error => Color::Red,
            ErrorLevel::Warning => Color::Yellow,
        }
    }
}
