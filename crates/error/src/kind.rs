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
// 2. Make sure that the index is in range 0..=9999.
// 3. Try to make similar error kinds have similar indexes.
//    - That's why there are gaps in the indexes: so that I can insert new error kinds.
//
// You can see the result of the macro expansion in `src/proc_macro.rs`.
// `ErrorKind` implements 1 method: `fn index(&self) -> u16;`.
// `ErrorKind` also implements `Endec`.
// `ErrorLevel` implements 1 method: `fn from_error_kind(k: &ErrorKind) -> Self;`.
error_kinds!(
    // error variant,                                              index,    Error | Warning
    (InvalidNumberLiteral,                                             0,    Error),
    (InvalidStringLiteralPrefix(Vec<u8>),                              5,    Error),
    (EmptyIdent,                                                      10,    Error),
    (InvalidCharacterInIdent(char),                                   15,    Error),
    (WrongNumberOfQuotesInRawStringLiteral,                           20,    Error),
    (UnterminatedStringLiteral,                                       25,    Error),
    (NotAllowedCharInFormattedString(u8),                             30,    Error),
    (UnmatchedBraceInFormattedString,                                 35,    Error),
    (EmptyBraceInFormattedString,                                     40,    Error),
    (DotDotDot,                                                       45,    Error),
    (InvalidCharLiteral,                                              50,    Error),
    (InvalidCharLiteralPrefix(Vec<u8>),                               55,    Error),
    (UnterminatedCharLiteral,                                         60,    Error),
    (InvalidByteLiteral,                                              65,    Error),
    (InvalidEscape,                                                   70,    Error),
    (EmptyCharLiteral,                                                75,    Error),
    (UnterminatedBlockComment,                                        80,    Error),
    (InvalidUtf8,                                                     85,    Error),
    (InvalidUnicodeCharacter,                                         90,    Error),
    (InvalidUnicodeEscape,                                            95,    Error),
    (UnmatchedGroup { expected: u8, got: u8 },                       100,    Error),

    // You can use up to 127 quotes for opening.
    // If a literal is opened with N quotes, it has to be closed with the same number of quotes.
    (TooManyQuotes,                                                  105,    Error),

    (UnclosedDelimiter(u8),                                          110,    Error),
    (UnexpectedToken { expected: ErrorToken, got: ErrorToken },      115,    Error),
    (UnexpectedEof { expected: ErrorToken },                         120,    Error),

    // It's like UnexpectedEof, but an end of a group (parenthesis, braces or brackets).
    (UnexpectedEog { expected: ErrorToken },                         125,    Error),

    (MissingDocComment,                                              130,    Error),
    (DocCommentNotAllowed,                                           135,    Error),
    (DanglingDocComment,                                             136,    Error),
    (ModuleDocCommentNotAtTop,                                       140,    Error),
    (MissingDecorator(InternedString),                               145,    Error),
    (DecoratorNotAllowed,                                            150,    Error),
    (DanglingDecorator,                                              151,    Error),
    (UnexpectedDecorator(InternedString),                            155,    Error),
    (ModuleDecoratorNotAtTop,                                        160,    Error),
    (MissingVisibility,                                              165,    Error),
    (CannotBePublic,                                                 170,    Error),
    (DanglingVisibility,                                             171,    Error),
    (FunctionWithoutBody,                                            175,    Error),
    (BlockWithoutValue,                                              180,    Error),
    (StructWithoutField,                                             185,    Error),
    (EmptyCurlyBraceBlock,                                           190,    Error),
    (AmbiguousCurlyBraces,                                           191,    Error),
    (PositionalArgAfterKeywordArg,                                   195,    Error),
    (NonDefaultValueAfterDefaultValue,                               200,    Error),
    (CannotDeclareInlineModule,                                      205,    Error),
    (InclusiveRangeWithNoEnd,                                        210,    Error),
    (MultipleRestPatterns,                                           215,    Error),
    (DifferentNameBindingsInOrPattern,                               220,    Error),
    (InvalidFnType,                                                  225,    Error),
    (EmptyMatchStatement,                                            230,    Error),
    (RedundantDecorator(InternedString),                             235,    Error),

    // TODO: suggest similar names
    // TODO: tell what it's trying to decorate
    (InvalidDecorator(InternedString),                               240,    Error),

    (MissingDecoratorArgument { expected: usize, got: usize },       245,    Error),
    (UnexpectedDecoratorArgument { expected: usize, got: usize },    250,    Error),
    (WrongNumberOfLangItemGenerics { lang_items: usize, generic_params: usize },    255,    Error),
    (CannotEvaluateConst,                                            260,    Error),

    // syntax errors in patterns
    (InvalidRangePattern,                                            265,    Error),
    (InvalidConcatPattern,                                           270,    Error),
    (CannotBindName(InternedString),                                 275,    Error),
    (CannotApplyInfixOpToMultipleBindings,                           280,    Error),
    (CannotApplyInfixOpToBinding,                                    285,    Error),
    (CannotAnnotateType,                                             290,    Error),
    (RedundantNameBinding(InternedString, InternedString),           295,    Error),
    (UnsupportedInfixOpInPattern(InfixOp),                           300,    Error),

    // TODO: more context!
    (NameCollision { name: InternedString, kind: NameCollisionKind },   305,    Error),

    (CyclicLet { names: Vec<InternedString> },                       310,    Error),
    (CyclicAlias { names: Vec<InternedString> },                     315,    Error),
    (DollarOutsidePipeline,                                          320,    Error),
    (DisconnectedPipeline,                                           325,    Error),

    // TODO: more context!
    // TODO: suggest similar names
    (UndefinedName(InternedString),                                  330,    Error),

    (EnumVariantInTypeAnnot,                                         335,    Error),
    (KeywordArgumentRepeated(InternedString),                        340,    Error),
    (KeywordArgumentNotAllowed,                                      345,    Error),
    (AliasResolveRecursionLimitReached,                              350,    Error),
    (MissingTypeParameter { expected: usize, got: usize },           355,    Error),
    (UnexpectedTypeParameter { expected: usize, got: usize },        360,    Error),
    (MissingKeywordArgument(InternedString),                         366,    Error),

    // TODO: more context!
    // TODO: suggest similar names
    (InvalidKeywordArgument(InternedString),                         370,    Error),

    (MissingFunctionParameter { expected: usize, got: usize },       375,    Error),
    (UnexpectedFunctionParameter { expected: usize, got: usize },    380,    Error),
    (StructFieldRepeated(InternedString),                            385,    Error),
    (MissingStructFields { struct_name: InternedString, missing_fields: Vec<InternedString> }, 390,    Error),
    (InvalidStructFields { struct_name: InternedString, invalid_fields: Vec<InternedString> }, 395,    Error),

    (CannotAssociateItem,                                            398,    Error),
    (TooGeneralToAssociateItem,                                      399,    Error),
    (NotType { id: InternedString, but: NotXBut },                   400,    Error),
    (NotCallable { r#type: String },                                 404,    Error),
    (NotStruct { id: InternedString, but: NotXBut },                 405,    Error),
    (NotExpr { id: InternedString, but: NotXBut },                   406,    Error),
    (NotPolyGeneric { id: Option<IdentWithOrigin> },                 410,    Error),

    // Type errors from here.
    // Type errors are generated by `inter-mir` crate, and the crate uses its own data types to
    // represent types. But this crate cannot depend on `inter-mir`, so those types are converted
    // to string.
    (UnexpectedType { expected: String, got: String },                   415,    Error),
    (CannotInferType { id: Option<InternedString>, is_return: bool },    420,    Error),
    (PartiallyInferedType { id: Option<InternedString>, r#type: String, is_return: bool }, 425,    Error),
    (CannotInferGenericType { id: Option<String> },                      430,    Error),
    (PartiallyInferedGenericType { id: Option<String>, r#type: String }, 435,    Error),
    (CannotApplyInfixOp { op: InfixOp, arg_types: Vec<String> },         440,    Error),
    (CannotSpecializePolyGeneric { num_candidates: usize },              445,    Error),
    (ImpureCallInPureContext,                                            450,    Error),

    // TODO: tell what's missing
    (NonExhaustiveArms,                                                  455,    Error),

    (MultipleModuleFiles { module: ModulePath, found_files: Vec<String> },    460,    Error),
    (ModuleFileNotFound { module: ModulePath, candidates: Vec<String> },      465,    Error),
    (LibFileNotFound,                                                470,    Error),

    (SelfParamWithTypeAnnot,                                         475,    Error),
    (AssociatedFuncWithoutSelfParam,                                 480,    Error),

    // Warnings from here
    (UnusedNames { names: Vec<InternedString>, kind: NameKind },    5000,  Warning),
    (UnreachableMatchArm,                                           5005,  Warning),
    (NoImpureCallInImpureContext,                                   5010,  Warning),

    // Lints from here
    (FuncWithoutTypeAnnot,                                          8000,  Lint),
    (LetWithoutTypeAnnot,                                           8005,  Lint),
    (FieldWithoutTypeAnnot,                                         8010,  Lint),
    (SelfParamNotNamedSelf,                                         8015,  Lint),

    // These are very special kinds of errors.
    // These are bugs in the compiler, not in the user's Sodigy code.
    // They use `id` field to distinguish themselves: so that we can easily Ctrl+Shift+F the id.
    (Todo { id: u32, message: String },                             9998,    Error),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NameCollisionKind {
    Block { is_top_level: bool },   // all items
    Enum,     // variants
    Func { params: bool, generics: bool },     // params and/or generics
    Pattern,  // name bindings
    Struct,   // fields
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NotXBut {
    Expr,
    Struct,
    Enum,
    Module,
    GenericParam,
}

impl NotXBut {
    pub fn with_article(&self) -> &'static str {
        match self {
            NotXBut::Expr => "an expression",
            NotXBut::Struct => "a struct",
            NotXBut::Enum => "an enum",
            NotXBut::Module => "a module",
            NotXBut::GenericParam => "a generic parameter",
        }
    }
}

impl From<NameKind> for NotXBut {
    fn from(k: NameKind) -> NotXBut {
        match k {
            NameKind::Let { .. } |
            NameKind::Func |
            NameKind::EnumVariant { .. } |
            NameKind::Alias |
            NameKind::Use |
            NameKind::FuncParam |
            NameKind::PatternNameBind |
            NameKind::Pipeline => NotXBut::Expr,
            NameKind::Struct => NotXBut::Struct,
            NameKind::Enum => NotXBut::Enum,
            NameKind::Module => NotXBut::Module,
            NameKind::GenericParam => NotXBut::GenericParam,
        }
    }
}
