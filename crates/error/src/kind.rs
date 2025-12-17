use crate::{ErrorLevel, ErrorToken};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error_gen::error_kinds;
use sodigy_file::{GetFilePathError, ModulePath};
use sodigy_name_analysis::{IdentWithOrigin, NameKind};
use sodigy_string::InternedString;
use sodigy_token::InfixOp;

mod render;

// It derives Clone, Debug, Eq, Hash and PartialEq.
//
// An error kind's index is not just for endec, but also for documentation (WIP).
// When you add an error kind and give an index to it:
// 1. Make sure that the index has never been used before.
// 2. Make sure that the index is in range 0..65536.
// 3. Try to make similar error kinds have similar indexes.
//    - That's why there are gaps in the indexes: so that I can insert new error kinds.
//
// You can see the result of the macro expansion in `src/proc_macro.rs`.
error_kinds!(
    // error variant,                                              index,    Error | Warning
    (InvalidNumberLiteral,                                             0,    Error),
    (InvalidStringLiteralPrefix(Vec<u8>),                              5,    Error),
    (InvalidCharacterInIdent(char),                                   10,    Error),
    (WrongNumberOfQuotesInRawStringLiteral,                           15,    Error),
    (UnterminatedStringLiteral,                                       20,    Error),
    (NotAllowedCharInFormattedString(u8),                             25,    Error),
    (UnmatchedBraceInFormattedString,                                 30,    Error),
    (EmptyBraceInFormattedString,                                     35,    Error),
    (DotDotDot,                                                       40,    Error),
    (InvalidCharLiteral,                                              45,    Error),
    (InvalidCharLiteralPrefix(Vec<u8>),                               50,    Error),
    (UnterminatedCharLiteral,                                         55,    Error),
    (InvalidByteLiteral,                                              60,    Error),
    (InvalidEscape,                                                   65,    Error),
    (EmptyCharLiteral,                                                70,    Error),
    (UnterminatedBlockComment,                                        75,    Error),
    (InvalidUtf8,                                                     80,    Error),
    (InvalidUnicodeCharacter,                                         85,    Error),
    (InvalidUnicodeEscape,                                            90,    Error),
    (UnmatchedGroup { expected: u8, got: u8 },                        95,    Error),

    // You can use up to 127 quotes for opening.
    // If a literal is opened with N quotes, it has to be closed with the same number of quotes.
    (TooManyQuotes,                                                  100,    Error),

    (UnclosedDelimiter(u8),                                          105,    Error),
    (UnexpectedToken { expected: ErrorToken, got: ErrorToken },      110,    Error),
    (UnexpectedEof { expected: ErrorToken },                         115,    Error),

    // It's like UnexpectedEof, but an end of a group (parenthesis, braces or brackets).
    (UnexpectedEog { expected: ErrorToken },                         120,    Error),

    (MissingDocComment,                                              125,    Error),
    (DocCommentNotAllowed,                                           130,    Error),
    (ModuleDocCommentNotAtTop,                                       135,    Error),
    (MissingDecorator(InternedString),                               140,    Error),
    (DecoratorNotAllowed,                                            145,    Error),
    (UnexpectedDecorator(InternedString),                            150,    Error),
    (ModuleDecoratorNotAtTop,                                        155,    Error),
    (MissingVisibility,                                              160,    Error),
    (CannotBePublic,                                                 165,    Error),
    (FunctionWithoutBody,                                            170,    Error),
    (BlockWithoutValue,                                              175,    Error),
    (StructWithoutField,                                             180,    Error),
    (EmptyCurlyBraceBlock,                                           185,    Error),
    (PositionalArgAfterKeywordArg,                                   190,    Error),
    (NonDefaultValueAfterDefaultValue,                               195,    Error),
    (CannotDeclareInlineModule,                                      200,    Error),
    (InclusiveRangeWithNoEnd,                                        205,    Error),
    (MultipleDotDotsInPattern,                                       210,    Error),
    (DifferentNameBindingsInOrPattern,                               215,    Error),
    (InvalidFnType,                                                  220,    Error),
    (EmptyMatchStatement,                                            225,    Error),
    (RedundantDecorator(InternedString),                             230,    Error),

    // TODO: suggest similar names
    // TODO: tell what it's trying to decorate
    (InvalidDecorator(InternedString),                               235,    Error),

    (MissingDecoratorArgument { expected: usize, got: usize },       240,    Error),
    (UnexpectedDecoratorArgument { expected: usize, got: usize },    245,    Error),
    (WrongNumberOfLangItemGenerics { lang_items: usize, generic_def: usize },    250,    Error),

    // syntax errors in patterns
    (InvalidRangePattern,                                            255,    Error),
    (CannotBindNameToAnotherName(InternedString),                    260,    Error),
    (CannotBindNameToConstant(InternedString),                       265,    Error),
    (CannotAnnotateType,                                             270,    Error),
    (RedundantNameBinding(InternedString, InternedString),           275,    Error),
    (CannotEvaluateConstPattern,                                     280,    Error),

    // TODO: more context!
    (NameCollision { name: InternedString },                         285,    Error),

    (CyclicLet { names: Vec<InternedString> },                       290,    Error),
    (CyclicAlias { names: Vec<InternedString> },                     295,    Error),
    (DollarOutsidePipeline,                                          300,    Error),
    (DisconnectedPipeline,                                           305,    Error),

    // TODO: more context!
    // TODO: suggest similar names
    (UndefinedName(InternedString),                                  310,    Error),

    (EnumVariantInTypeAnnotation,                                    315,    Error),
    (KeywordArgumentRepeated(InternedString),                        320,    Error),
    (KeywordArgumentNotAllowed,                                      325,    Error),
    (AliasResolveRecursionLimitReached,                              330,    Error),
    (MissingTypeParameter { expected: usize, got: usize },           335,    Error),
    (UnexpectedTypeParameter { expected: usize, got: usize },        340,    Error),
    (MissingKeywordArgument(InternedString),                         345,    Error),

    // TODO: more context!
    // TODO: suggest similar names
    (InvalidKeywordArgument(InternedString),                         350,    Error),

    (MissingFunctionParameter { expected: usize, got: usize },       355,    Error),
    (UnexpectedFunctionParameter { expected: usize, got: usize },    360,    Error),
    (StructFieldRepeated(InternedString),                            365,    Error),
    (MissingStructField(InternedString),                             370,    Error),

    // TODO: suggest similar names
    (InvalidStructField(InternedString),                             375,    Error),

    (DependentTypeNotAllowed,                                        380,    Error),
    (NotStruct { id: Option<IdentWithOrigin> },                      385,    Error),
    (NotPolyGeneric { id: Option<IdentWithOrigin> },                 390,    Error),

    // Type errors from here.
    // Type errors are generated by `mir-type` crate, and the crate uses its own data types to
    // represent types. But this crate cannot depend on `mir-type`, so those types are converted
    // to string.
    (UnexpectedType { expected: String, got: String },               395,    Error),
    (CannotInferType { id: Option<InternedString> },                 400,    Error),
    (PartiallyInferedType { id: Option<InternedString>, r#type: String },    405,    Error),
    (CannotInferGenericType { id: Option<String> },                  410,    Error),
    (PartiallyInferedGenericType { id: Option<String>, r#type: String },     415,    Error),
    (CannotApplyInfixOp { op: InfixOp, arg_types: Vec<String> },     420,    Error),
    (CannotSpecializePolyGeneric { num_candidates: usize },          425,    Error),

    (MultipleModuleFiles { module: ModulePath, found_files: Vec<String> },    430,    Error),
    (ModuleFileNotFound { module: ModulePath, candidates: Vec<String> },      435,    Error),
    (LibFileNotFound,                                                440,    Error),

    // Warnings from here
    (UnusedNames { names: Vec<InternedString>, kind: NameKind },    5000,  Warning),

    // These are very special kinds of errors.
    // These are bugs in the compiler, not in the user's Sodigy code.
    // They use `id` field to distinguish themselves: so that we can easily Ctrl+Shift+F the id.
    (Todo { id: u32 },                                              9998,    Error),
    (InternalCompilerError { id: u32 },                             9999,    Error),
);

impl From<GetFilePathError> for ErrorKind {
    fn from(e: GetFilePathError) -> ErrorKind {
        if e.is_std && e.found_files.is_empty() {
            ErrorKind::LibFileNotFound
        }

        else if e.found_files.is_empty() {
            ErrorKind::ModuleFileNotFound {
                module: e.module_path.clone(),
                candidates: e.candidates.clone(),
            }
        }

        else if e.found_files.len() > 1 {
            ErrorKind::MultipleModuleFiles {
                module: e.module_path.clone(),
                found_files: e.found_files.clone(),
            }
        }

        else {
            unreachable!()
        }
    }
}
