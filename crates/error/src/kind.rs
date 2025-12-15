use crate::ErrorToken;
use sodigy_error_gen::error_kinds;
use sodigy_file::{GetFilePathError, ModulePath};
use sodigy_name_analysis::NameKind;
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
error_kinds!(
    //                               error kind,                                    fields,  index,   Error | Warning
    (                      InvalidNumberLiteral,                                         _,      0,      Error),
    (                InvalidStringLiteralPrefix,                                         _,      5,      Error),
    (                   InvalidCharacterInIdent,                                    (char),     10,      Error),
    (     WrongNumberOfQuotesInRawStringLiteral,                                         _,     15,      Error),
    (                 UnterminatedStringLiteral,                                         _,     20,      Error),
    (           NotAllowedCharInFormattedString,                                      (u8),     25,      Error),
    (           UnmatchedBraceInFormattedString,                                         _,     30,      Error),
    (               EmptyBraceInFormattedString,                                         _,     35,      Error),
    (                                 DotDotDot,                                         _,     40,      Error),
    (                        InvalidCharLiteral,                                         _,     45,      Error),
    (                  InvalidCharLiteralPrefix,                                 (Vec<u8>),     50,      Error),
    (                   UnterminatedCharLiteral,                                         _,     55,      Error),
    (                        InvalidByteLiteral,                                         _,     60,      Error),
    (                             InvalidEscape,                                         _,     65,      Error),
    (                          EmptyCharLiteral,                                         _,     70,      Error),
    (                  UnterminatedBlockComment,                                         _,     75,      Error),
    (                               InvalidUtf8,                                         _,     80,      Error),
    (                   InvalidUnicodeCharacter,                                         _,     85,      Error),
    (                      InvalidUnicodeEscape,                                         _,     90,      Error),
    (                            UnmatchedGroup,                 { expected: u8, got: u8 },     95,      Error),

    // You can use up to 127 quotes for opening.
    // If a literal is opened with N quotes, it has to be closed with the same number of quotes.
    (                             TooManyQuotes,                                         _,    100,      Error),

    (                         UnclosedDelimiter,                                      (u8),    105,      Error),
    (                           UnexpectedToken, { expected: ErrorToken, got: ErrorToken },    110,      Error),
    (                             UnexpectedEof,                  { expected: ErrorToken },    115,      Error),

    // It's like UnexpectedEof, but an end of a group (parenthesis, braces or brackets).
    (                             UnexpectedEog,                  { expected: ErrorToken },    120,      Error),

    (                         MissingDocComment,                                         _,    125,      Error),
    (                      DocCommentNotAllowed,                                         _,    130,      Error),
    (                  ModuleDocCommentNotAtTop,                                         _,    135,      Error),
    (                          MissingDecorator,                          (InternedString),    140,      Error),
    (                       DecoratorNotAllowed,                                         _,    145,      Error),
    (                       UnexpectedDecorator,                          (InternedString),    150,      Error),
    (                   ModuleDecoratorNotAtTop,                                         _,    155,      Error),
    (                         MissingVisibility,                                         _,    160,      Error),
    (                            CannotBePublic,                                         _,    165,      Error),
    (                       FunctionWithoutBody,                                         _,    170,      Error),
    (                         BlockWithoutValue,                                         _,    175,      Error),
    (                        StructWithoutField,                                         _,    180,      Error),
    (                      EmptyCurlyBraceBlock,                                         _,    185,      Error),
    (              PositionalArgAfterKeywordArg,                                         _,    190,      Error),
    (          NonDefaultValueAfterDefaultValue,                                         _,    195,      Error),
    (                 CannotDeclareInlineModule,                                         _,    200,      Error),
    (                   InclusiveRangeWithNoEnd,                                         _,    205,      Error),
    (                  MultipleDotDotsInPattern,                                         _,    210,      Error),
    (          DifferentNameBindingsInOrPattern,                                         _,    215,      Error),
    (                             InvalidFnType,                                         _,    220,      Error),
    (                       EmptyMatchStatement,                                         _,    225,      Error),
    (                        RedundantDecorator,                          (InternedString),    230,      Error),

    // TODO: suggest similar names
    // TODO: tell what it's trying to decorate
    (                          InvalidDecorator,                          (InternedString),    235,      Error),

    (                  MissingDecoratorArgument,           { expected: usize, got: usize },    240,      Error),
    (               UnexpectedDecoratorArgument,           { expected: usize, got: usize },    245,      Error),
    (             WrongNumberOfLangItemGenerics, { lang_items: usize, generic_def: usize },    250,      Error),

    // syntax errors in patterns
    (                       InvalidRangePattern,                                         _,    255,      Error),
    (               CannotBindNameToAnotherName,                          (InternedString),    260,      Error),
    (                  CannotBindNameToConstant,                          (InternedString),    265,      Error),
    (                        CannotAnnotateType,                                         _,    270,      Error),
    (                      RedundantNameBinding,          (InternedString, InternedString),    275,      Error),
    (                CannotEvaluateConstPattern,                                         _,    280,      Error),

    // TODO: more context!
    (                             NameCollision,                  { name: InternedString },    285,      Error),

    (                                 CyclicLet,            { names: Vec<InternedString> },    290,      Error),
    (                               CyclicAlias,            { names: Vec<InternedString> },    295,      Error),

    // TODO: more context!
    // TODO: suggest similar names
    (                             UndefinedName,                          (InternedString),    300,      Error),

    (                   KeywordArgumentRepeated,                          (InternedString),    305,      Error),
    (                 KeywordArgumentNotAllowed,                                         _,    310,      Error),
    (         AliasResolveRecursionLimitReached,                                         _,    315,      Error),
    (                      MissingTypeParameter,           { expected: usize, got: usize },    320,      Error),
    (                   UnexpectedTypeParameter,           { expected: usize, got: usize },    325,      Error),
    (                    MissingKeywordArgument,                          (InternedString),    330,      Error),

    // TODO: more context!
    // TODO: suggest similar names
    (                    InvalidKeywordArgument,                          (InternedString),    335,      Error),

    (                  MissingFunctionParameter,           { expected: usize, got: usize },    340,      Error),
    (               UnexpectedFunctionParameter,           { expected: usize, got: usize },    345,      Error),
    (                       StructFieldRepeated,                          (InternedString),    350,      Error),
    (                        MissingStructField,                          (InternedString),    355,      Error),

    // TODO: suggest similar names
    (                        InvalidStructField,                          (InternedString),    360,      Error),

    (                   DependentTypeNotAllowed,                                         _,    365,      Error),

    // Type errors from here.
    // Type errors are generated by `mir-type` crate, and the crate uses its own data types to
    // represent types. But this crate cannot depend on `mir-type`, so those types are converted
    // to string.
    (                            UnexpectedType,         { expected: String, got: String },    370,      Error),
    (                           CannotInferType,            { id: Option<InternedString> },    375,      Error),
    (                 PartiallyInferedType, { id: Option<InternedString>, r#type: String },    380,      Error),
    (                    CannotInferGenericType,            { id: Option<InternedString> },    385,      Error),
    (               PartiallyInferedGenericType,    { id: Option<String>, r#type: String },    390,      Error),
    (                        CannotApplyInfixOp,   { op: InfixOp, arg_types: Vec<String> },    395,      Error),
    (               CannotSpecializePolyGeneric,                 { num_candidates: usize },    400,      Error),

    (                MultipleModuleFiles, { module: ModulePath, found_files: Vec<String> },    405,      Error),
    (                 ModuleFileNotFound,  { module: ModulePath, candidates: Vec<String> },    410,      Error),
    (                           LibFileNotFound,                                         _,    415,      Error),

    // These are very special kinds of errors.
    // These are bugs in the compiler, not in the user's Sodigy code.
    // They use `id` field to distinguish themselves: so that we can easily Ctrl+Shift+F the id.
    (                                      Todo,                               { id: u32 },   9998,      Error),
    (                     InternalCompilerError,                               { id: u32 },   9999,      Error),

    // Warnings from here
    (                          UnusedNames, { names: Vec<InternedString>, kind: NameKind },   5000,    Warning),
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
