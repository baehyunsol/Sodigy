# [derive(Clone, Debug, Eq, Hash, PartialEq)] pub enum ErrorKind {
    InvalidNumberLiteral, InvalidStringLiteralPrefix(Vec<u8>), EmptyIdent,
    InvalidCharacterInIdent(char), WrongNumberOfQuotesInRawStringLiteral,
    UnterminatedStringLiteral, NotAllowedCharInFormattedString(u8),
    UnmatchedBraceInFormattedString, EmptyBraceInFormattedString, DotDotDot,
    InvalidCharLiteral, InvalidCharLiteralPrefix(Vec<u8>),
    UnterminatedCharLiteral, InvalidByteLiteral, InvalidEscape,
    EmptyCharLiteral, UnterminatedBlockComment, InvalidUtf8,
    InvalidUnicodeCharacter, InvalidUnicodeEscape, UnmatchedGroup
    { expected: u8, got: u8 }, TooManyQuotes, UnclosedDelimiter(u8),
    UnexpectedToken { expected: ErrorToken, got: ErrorToken }, UnexpectedEof
    { expected: ErrorToken }, UnexpectedEog { expected: ErrorToken },
    MissingDocComment, DocCommentNotAllowed, DanglingDocComment,
    ModuleDocCommentNotAtTop, MissingDecorator(InternedString),
    DecoratorNotAllowed, DanglingDecorator,
    UnexpectedDecorator(InternedString), ModuleDecoratorNotAtTop,
    MissingVisibility, CannotBePublic, DanglingVisibility,
    FunctionWithoutBody, BlockWithoutValue, StructWithoutField,
    EmptyCurlyBraceBlock, PositionalArgAfterKeywordArg,
    NonDefaultValueAfterDefaultValue, CannotDeclareInlineModule,
    InclusiveRangeWithNoEnd, MultipleRestPatterns,
    DifferentNameBindingsInOrPattern, InvalidFnType, EmptyMatchStatement,
    RedundantDecorator(InternedString), InvalidDecorator(InternedString),
    MissingDecoratorArgument { expected: usize, got: usize },
    UnexpectedDecoratorArgument { expected: usize, got: usize },
    WrongNumberOfLangItemGenerics { lang_items: usize, generic_def: usize },
    CannotEvaluateConst, InvalidRangePattern, InvalidConcatPattern,
    CannotBindName(InternedString), CannotApplyInfixOpToMultipleBindings,
    CannotApplyInfixOpToBinding, CannotAnnotateType,
    RedundantNameBinding(InternedString, InternedString),
    UnsupportedInfixOpInPattern(InfixOp), NameCollision
    { name: InternedString, kind: NameCollisionKind }, CyclicLet
    { names: Vec<InternedString> }, CyclicAlias
    { names: Vec<InternedString> }, DollarOutsidePipeline,
    DisconnectedPipeline, UndefinedName(InternedString),
    EnumVariantInTypeAnnotation, KeywordArgumentRepeated(InternedString),
    KeywordArgumentNotAllowed, AliasResolveRecursionLimitReached,
    MissingTypeParameter { expected: usize, got: usize },
    UnexpectedTypeParameter { expected: usize, got: usize },
    MissingKeywordArgument(InternedString),
    InvalidKeywordArgument(InternedString), MissingFunctionParameter
    { expected: usize, got: usize }, UnexpectedFunctionParameter
    { expected: usize, got: usize }, StructFieldRepeated(InternedString),
    MissingStructFields
    { struct_name: InternedString, missing_fields: Vec<InternedString> },
    InvalidStructFields
    { struct_name: InternedString, invalid_fields: Vec<InternedString> },
    CannotAssociateItem, TooGeneralToAssociateItem, DependentTypeNotAllowed,
    NotCallable { r#type: String }, NotStruct { id: Option<IdentWithOrigin> },
    NotExpr { id: InternedString, kind: NotExprBut }, NotPolyGeneric
    { id: Option<IdentWithOrigin> }, UnexpectedType
    { expected: String, got: String }, CannotInferType
    { id: Option<InternedString>, is_return: bool }, PartiallyInferedType
    { id: Option<InternedString>, r#type: String, is_return: bool },
    CannotInferGenericType { id: Option<String> }, PartiallyInferedGenericType
    { id: Option<String>, r#type: String }, CannotApplyInfixOp
    { op: InfixOp, arg_types: Vec<String> }, CannotSpecializePolyGeneric
    { num_candidates: usize }, ImpureCallInPureContext, NonExhaustiveArms,
    MultipleModuleFiles { module: ModulePath, found_files: Vec<String> },
    ModuleFileNotFound { module: ModulePath, candidates: Vec<String> },
    LibFileNotFound, SelfParamWithTypeAnnotation,
    AssociatedFuncWithoutSelfParam, UnusedNames
    { names: Vec<InternedString>, kind: NameKind }, UnreachableMatchArm,
    NoImpureCallInImpureContext, FuncWithoutTypeAnnotation,
    LetWithoutTypeAnnotation, FieldWithoutTypeAnnotation,
    SelfParamNotNamedSelf, Todo { id: u32, message: String },
    InternalCompilerError { id: u32 },
} impl ErrorKind {
    pub fn index(& self) -> u16
    {
        match self
        {
            ErrorKind :: InvalidNumberLiteral => 0u16, ErrorKind ::
            InvalidStringLiteralPrefix(_,) => 5u16, ErrorKind :: EmptyIdent =>
            10u16, ErrorKind :: InvalidCharacterInIdent(_,) => 15u16,
            ErrorKind :: WrongNumberOfQuotesInRawStringLiteral => 20u16,
            ErrorKind :: UnterminatedStringLiteral => 25u16, ErrorKind ::
            NotAllowedCharInFormattedString(_,) => 30u16, ErrorKind ::
            UnmatchedBraceInFormattedString => 35u16, ErrorKind ::
            EmptyBraceInFormattedString => 40u16, ErrorKind :: DotDotDot =>
            45u16, ErrorKind :: InvalidCharLiteral => 50u16, ErrorKind ::
            InvalidCharLiteralPrefix(_,) => 55u16, ErrorKind ::
            UnterminatedCharLiteral => 60u16, ErrorKind :: InvalidByteLiteral
            => 65u16, ErrorKind :: InvalidEscape => 70u16, ErrorKind ::
            EmptyCharLiteral => 75u16, ErrorKind :: UnterminatedBlockComment
            => 80u16, ErrorKind :: InvalidUtf8 => 85u16, ErrorKind ::
            InvalidUnicodeCharacter => 90u16, ErrorKind ::
            InvalidUnicodeEscape => 95u16, ErrorKind :: UnmatchedGroup { .. }
            => 100u16, ErrorKind :: TooManyQuotes => 105u16, ErrorKind ::
            UnclosedDelimiter(_,) => 110u16, ErrorKind :: UnexpectedToken
            { .. } => 115u16, ErrorKind :: UnexpectedEof { .. } => 120u16,
            ErrorKind :: UnexpectedEog { .. } => 125u16, ErrorKind ::
            MissingDocComment => 130u16, ErrorKind :: DocCommentNotAllowed =>
            135u16, ErrorKind :: DanglingDocComment => 136u16, ErrorKind ::
            ModuleDocCommentNotAtTop => 140u16, ErrorKind ::
            MissingDecorator(_,) => 145u16, ErrorKind :: DecoratorNotAllowed
            => 150u16, ErrorKind :: DanglingDecorator => 151u16, ErrorKind ::
            UnexpectedDecorator(_,) => 155u16, ErrorKind ::
            ModuleDecoratorNotAtTop => 160u16, ErrorKind :: MissingVisibility
            => 165u16, ErrorKind :: CannotBePublic => 170u16, ErrorKind ::
            DanglingVisibility => 171u16, ErrorKind :: FunctionWithoutBody =>
            175u16, ErrorKind :: BlockWithoutValue => 180u16, ErrorKind ::
            StructWithoutField => 185u16, ErrorKind :: EmptyCurlyBraceBlock =>
            190u16, ErrorKind :: PositionalArgAfterKeywordArg => 195u16,
            ErrorKind :: NonDefaultValueAfterDefaultValue => 200u16, ErrorKind
            :: CannotDeclareInlineModule => 205u16, ErrorKind ::
            InclusiveRangeWithNoEnd => 210u16, ErrorKind ::
            MultipleRestPatterns => 215u16, ErrorKind ::
            DifferentNameBindingsInOrPattern => 220u16, ErrorKind ::
            InvalidFnType => 225u16, ErrorKind :: EmptyMatchStatement =>
            230u16, ErrorKind :: RedundantDecorator(_,) => 235u16, ErrorKind
            :: InvalidDecorator(_,) => 240u16, ErrorKind ::
            MissingDecoratorArgument { .. } => 245u16, ErrorKind ::
            UnexpectedDecoratorArgument { .. } => 250u16, ErrorKind ::
            WrongNumberOfLangItemGenerics { .. } => 255u16, ErrorKind ::
            CannotEvaluateConst => 260u16, ErrorKind :: InvalidRangePattern =>
            265u16, ErrorKind :: InvalidConcatPattern => 270u16, ErrorKind ::
            CannotBindName(_,) => 275u16, ErrorKind ::
            CannotApplyInfixOpToMultipleBindings => 280u16, ErrorKind ::
            CannotApplyInfixOpToBinding => 285u16, ErrorKind ::
            CannotAnnotateType => 290u16, ErrorKind ::
            RedundantNameBinding(_, _,) => 295u16, ErrorKind ::
            UnsupportedInfixOpInPattern(_,) => 300u16, ErrorKind ::
            NameCollision { .. } => 305u16, ErrorKind :: CyclicLet { .. } =>
            310u16, ErrorKind :: CyclicAlias { .. } => 315u16, ErrorKind ::
            DollarOutsidePipeline => 320u16, ErrorKind :: DisconnectedPipeline
            => 325u16, ErrorKind :: UndefinedName(_,) => 330u16, ErrorKind ::
            EnumVariantInTypeAnnotation => 335u16, ErrorKind ::
            KeywordArgumentRepeated(_,) => 340u16, ErrorKind ::
            KeywordArgumentNotAllowed => 345u16, ErrorKind ::
            AliasResolveRecursionLimitReached => 350u16, ErrorKind ::
            MissingTypeParameter { .. } => 355u16, ErrorKind ::
            UnexpectedTypeParameter { .. } => 360u16, ErrorKind ::
            MissingKeywordArgument(_,) => 366u16, ErrorKind ::
            InvalidKeywordArgument(_,) => 370u16, ErrorKind ::
            MissingFunctionParameter { .. } => 375u16, ErrorKind ::
            UnexpectedFunctionParameter { .. } => 380u16, ErrorKind ::
            StructFieldRepeated(_,) => 385u16, ErrorKind ::
            MissingStructFields { .. } => 390u16, ErrorKind ::
            InvalidStructFields { .. } => 395u16, ErrorKind ::
            CannotAssociateItem => 398u16, ErrorKind ::
            TooGeneralToAssociateItem => 399u16, ErrorKind ::
            DependentTypeNotAllowed => 400u16, ErrorKind :: NotCallable { .. }
            => 404u16, ErrorKind :: NotStruct { .. } => 405u16, ErrorKind ::
            NotExpr { .. } => 406u16, ErrorKind :: NotPolyGeneric { .. } =>
            410u16, ErrorKind :: UnexpectedType { .. } => 415u16, ErrorKind ::
            CannotInferType { .. } => 420u16, ErrorKind ::
            PartiallyInferedType { .. } => 425u16, ErrorKind ::
            CannotInferGenericType { .. } => 430u16, ErrorKind ::
            PartiallyInferedGenericType { .. } => 435u16, ErrorKind ::
            CannotApplyInfixOp { .. } => 440u16, ErrorKind ::
            CannotSpecializePolyGeneric { .. } => 445u16, ErrorKind ::
            ImpureCallInPureContext => 450u16, ErrorKind :: NonExhaustiveArms
            => 455u16, ErrorKind :: MultipleModuleFiles { .. } => 460u16,
            ErrorKind :: ModuleFileNotFound { .. } => 465u16, ErrorKind ::
            LibFileNotFound => 470u16, ErrorKind ::
            SelfParamWithTypeAnnotation => 475u16, ErrorKind ::
            AssociatedFuncWithoutSelfParam => 480u16, ErrorKind :: UnusedNames
            { .. } => 5000u16, ErrorKind :: UnreachableMatchArm => 5005u16,
            ErrorKind :: NoImpureCallInImpureContext => 5010u16, ErrorKind ::
            FuncWithoutTypeAnnotation => 8000u16, ErrorKind ::
            LetWithoutTypeAnnotation => 8005u16, ErrorKind ::
            FieldWithoutTypeAnnotation => 8010u16, ErrorKind ::
            SelfParamNotNamedSelf => 8015u16, ErrorKind :: Todo { .. } =>
            9998u16, ErrorKind :: InternalCompilerError { .. } => 9999u16,
        }
    }
} impl ErrorLevel {
    pub fn from_error_kind(k : & ErrorKind) -> ErrorLevel
    {
        match k
        {
            ErrorKind :: InvalidNumberLiteral => ErrorLevel :: Error,
            ErrorKind :: InvalidStringLiteralPrefix(_,) => ErrorLevel ::
            Error, ErrorKind :: EmptyIdent => ErrorLevel :: Error, ErrorKind
            :: InvalidCharacterInIdent(_,) => ErrorLevel :: Error, ErrorKind
            :: WrongNumberOfQuotesInRawStringLiteral => ErrorLevel :: Error,
            ErrorKind :: UnterminatedStringLiteral => ErrorLevel :: Error,
            ErrorKind :: NotAllowedCharInFormattedString(_,) => ErrorLevel ::
            Error, ErrorKind :: UnmatchedBraceInFormattedString => ErrorLevel
            :: Error, ErrorKind :: EmptyBraceInFormattedString => ErrorLevel
            :: Error, ErrorKind :: DotDotDot => ErrorLevel :: Error, ErrorKind
            :: InvalidCharLiteral => ErrorLevel :: Error, ErrorKind ::
            InvalidCharLiteralPrefix(_,) => ErrorLevel :: Error, ErrorKind ::
            UnterminatedCharLiteral => ErrorLevel :: Error, ErrorKind ::
            InvalidByteLiteral => ErrorLevel :: Error, ErrorKind ::
            InvalidEscape => ErrorLevel :: Error, ErrorKind ::
            EmptyCharLiteral => ErrorLevel :: Error, ErrorKind ::
            UnterminatedBlockComment => ErrorLevel :: Error, ErrorKind ::
            InvalidUtf8 => ErrorLevel :: Error, ErrorKind ::
            InvalidUnicodeCharacter => ErrorLevel :: Error, ErrorKind ::
            InvalidUnicodeEscape => ErrorLevel :: Error, ErrorKind ::
            UnmatchedGroup { .. } => ErrorLevel :: Error, ErrorKind ::
            TooManyQuotes => ErrorLevel :: Error, ErrorKind ::
            UnclosedDelimiter(_,) => ErrorLevel :: Error, ErrorKind ::
            UnexpectedToken { .. } => ErrorLevel :: Error, ErrorKind ::
            UnexpectedEof { .. } => ErrorLevel :: Error, ErrorKind ::
            UnexpectedEog { .. } => ErrorLevel :: Error, ErrorKind ::
            MissingDocComment => ErrorLevel :: Error, ErrorKind ::
            DocCommentNotAllowed => ErrorLevel :: Error, ErrorKind ::
            DanglingDocComment => ErrorLevel :: Error, ErrorKind ::
            ModuleDocCommentNotAtTop => ErrorLevel :: Error, ErrorKind ::
            MissingDecorator(_,) => ErrorLevel :: Error, ErrorKind ::
            DecoratorNotAllowed => ErrorLevel :: Error, ErrorKind ::
            DanglingDecorator => ErrorLevel :: Error, ErrorKind ::
            UnexpectedDecorator(_,) => ErrorLevel :: Error, ErrorKind ::
            ModuleDecoratorNotAtTop => ErrorLevel :: Error, ErrorKind ::
            MissingVisibility => ErrorLevel :: Error, ErrorKind ::
            CannotBePublic => ErrorLevel :: Error, ErrorKind ::
            DanglingVisibility => ErrorLevel :: Error, ErrorKind ::
            FunctionWithoutBody => ErrorLevel :: Error, ErrorKind ::
            BlockWithoutValue => ErrorLevel :: Error, ErrorKind ::
            StructWithoutField => ErrorLevel :: Error, ErrorKind ::
            EmptyCurlyBraceBlock => ErrorLevel :: Error, ErrorKind ::
            PositionalArgAfterKeywordArg => ErrorLevel :: Error, ErrorKind ::
            NonDefaultValueAfterDefaultValue => ErrorLevel :: Error, ErrorKind
            :: CannotDeclareInlineModule => ErrorLevel :: Error, ErrorKind ::
            InclusiveRangeWithNoEnd => ErrorLevel :: Error, ErrorKind ::
            MultipleRestPatterns => ErrorLevel :: Error, ErrorKind ::
            DifferentNameBindingsInOrPattern => ErrorLevel :: Error, ErrorKind
            :: InvalidFnType => ErrorLevel :: Error, ErrorKind ::
            EmptyMatchStatement => ErrorLevel :: Error, ErrorKind ::
            RedundantDecorator(_,) => ErrorLevel :: Error, ErrorKind ::
            InvalidDecorator(_,) => ErrorLevel :: Error, ErrorKind ::
            MissingDecoratorArgument { .. } => ErrorLevel :: Error, ErrorKind
            :: UnexpectedDecoratorArgument { .. } => ErrorLevel :: Error,
            ErrorKind :: WrongNumberOfLangItemGenerics { .. } => ErrorLevel ::
            Error, ErrorKind :: CannotEvaluateConst => ErrorLevel :: Error,
            ErrorKind :: InvalidRangePattern => ErrorLevel :: Error, ErrorKind
            :: InvalidConcatPattern => ErrorLevel :: Error, ErrorKind ::
            CannotBindName(_,) => ErrorLevel :: Error, ErrorKind ::
            CannotApplyInfixOpToMultipleBindings => ErrorLevel :: Error,
            ErrorKind :: CannotApplyInfixOpToBinding => ErrorLevel :: Error,
            ErrorKind :: CannotAnnotateType => ErrorLevel :: Error, ErrorKind
            :: RedundantNameBinding(_, _,) => ErrorLevel :: Error, ErrorKind
            :: UnsupportedInfixOpInPattern(_,) => ErrorLevel :: Error,
            ErrorKind :: NameCollision { .. } => ErrorLevel :: Error,
            ErrorKind :: CyclicLet { .. } => ErrorLevel :: Error, ErrorKind ::
            CyclicAlias { .. } => ErrorLevel :: Error, ErrorKind ::
            DollarOutsidePipeline => ErrorLevel :: Error, ErrorKind ::
            DisconnectedPipeline => ErrorLevel :: Error, ErrorKind ::
            UndefinedName(_,) => ErrorLevel :: Error, ErrorKind ::
            EnumVariantInTypeAnnotation => ErrorLevel :: Error, ErrorKind ::
            KeywordArgumentRepeated(_,) => ErrorLevel :: Error, ErrorKind ::
            KeywordArgumentNotAllowed => ErrorLevel :: Error, ErrorKind ::
            AliasResolveRecursionLimitReached => ErrorLevel :: Error,
            ErrorKind :: MissingTypeParameter { .. } => ErrorLevel :: Error,
            ErrorKind :: UnexpectedTypeParameter { .. } => ErrorLevel ::
            Error, ErrorKind :: MissingKeywordArgument(_,) => ErrorLevel ::
            Error, ErrorKind :: InvalidKeywordArgument(_,) => ErrorLevel ::
            Error, ErrorKind :: MissingFunctionParameter { .. } => ErrorLevel
            :: Error, ErrorKind :: UnexpectedFunctionParameter { .. } =>
            ErrorLevel :: Error, ErrorKind :: StructFieldRepeated(_,) =>
            ErrorLevel :: Error, ErrorKind :: MissingStructFields { .. } =>
            ErrorLevel :: Error, ErrorKind :: InvalidStructFields { .. } =>
            ErrorLevel :: Error, ErrorKind :: CannotAssociateItem =>
            ErrorLevel :: Error, ErrorKind :: TooGeneralToAssociateItem =>
            ErrorLevel :: Error, ErrorKind :: DependentTypeNotAllowed =>
            ErrorLevel :: Error, ErrorKind :: NotCallable { .. } => ErrorLevel
            :: Error, ErrorKind :: NotStruct { .. } => ErrorLevel :: Error,
            ErrorKind :: NotExpr { .. } => ErrorLevel :: Error, ErrorKind ::
            NotPolyGeneric { .. } => ErrorLevel :: Error, ErrorKind ::
            UnexpectedType { .. } => ErrorLevel :: Error, ErrorKind ::
            CannotInferType { .. } => ErrorLevel :: Error, ErrorKind ::
            PartiallyInferedType { .. } => ErrorLevel :: Error, ErrorKind ::
            CannotInferGenericType { .. } => ErrorLevel :: Error, ErrorKind ::
            PartiallyInferedGenericType { .. } => ErrorLevel :: Error,
            ErrorKind :: CannotApplyInfixOp { .. } => ErrorLevel :: Error,
            ErrorKind :: CannotSpecializePolyGeneric { .. } => ErrorLevel ::
            Error, ErrorKind :: ImpureCallInPureContext => ErrorLevel ::
            Error, ErrorKind :: NonExhaustiveArms => ErrorLevel :: Error,
            ErrorKind :: MultipleModuleFiles { .. } => ErrorLevel :: Error,
            ErrorKind :: ModuleFileNotFound { .. } => ErrorLevel :: Error,
            ErrorKind :: LibFileNotFound => ErrorLevel :: Error, ErrorKind ::
            SelfParamWithTypeAnnotation => ErrorLevel :: Error, ErrorKind ::
            AssociatedFuncWithoutSelfParam => ErrorLevel :: Error, ErrorKind
            :: UnusedNames { .. } => ErrorLevel :: Warning, ErrorKind ::
            UnreachableMatchArm => ErrorLevel :: Warning, ErrorKind ::
            NoImpureCallInImpureContext => ErrorLevel :: Warning, ErrorKind ::
            FuncWithoutTypeAnnotation => ErrorLevel :: Lint, ErrorKind ::
            LetWithoutTypeAnnotation => ErrorLevel :: Lint, ErrorKind ::
            FieldWithoutTypeAnnotation => ErrorLevel :: Lint, ErrorKind ::
            SelfParamNotNamedSelf => ErrorLevel :: Lint, ErrorKind :: Todo
            { .. } => ErrorLevel :: Error, ErrorKind :: InternalCompilerError
            { .. } => ErrorLevel :: Error,
        }
    }
} impl Endec for ErrorKind {
    fn encode_impl(& self, buffer : & mut Vec < u8 >)
    {
        match self
        {
            ErrorKind :: InvalidNumberLiteral =>
            { buffer.push(0u8); buffer.push(0u8); }, ErrorKind ::
            InvalidStringLiteralPrefix(t0,) =>
            { buffer.push(0u8); buffer.push(5u8); t0.encode_impl(buffer); },
            ErrorKind :: EmptyIdent =>
            { buffer.push(0u8); buffer.push(10u8); }, ErrorKind ::
            InvalidCharacterInIdent(t0,) =>
            { buffer.push(0u8); buffer.push(15u8); t0.encode_impl(buffer); },
            ErrorKind :: WrongNumberOfQuotesInRawStringLiteral =>
            { buffer.push(0u8); buffer.push(20u8); }, ErrorKind ::
            UnterminatedStringLiteral =>
            { buffer.push(0u8); buffer.push(25u8); }, ErrorKind ::
            NotAllowedCharInFormattedString(t0,) =>
            { buffer.push(0u8); buffer.push(30u8); t0.encode_impl(buffer); },
            ErrorKind :: UnmatchedBraceInFormattedString =>
            { buffer.push(0u8); buffer.push(35u8); }, ErrorKind ::
            EmptyBraceInFormattedString =>
            { buffer.push(0u8); buffer.push(40u8); }, ErrorKind :: DotDotDot
            => { buffer.push(0u8); buffer.push(45u8); }, ErrorKind ::
            InvalidCharLiteral => { buffer.push(0u8); buffer.push(50u8); },
            ErrorKind :: InvalidCharLiteralPrefix(t0,) =>
            { buffer.push(0u8); buffer.push(55u8); t0.encode_impl(buffer); },
            ErrorKind :: UnterminatedCharLiteral =>
            { buffer.push(0u8); buffer.push(60u8); }, ErrorKind ::
            InvalidByteLiteral => { buffer.push(0u8); buffer.push(65u8); },
            ErrorKind :: InvalidEscape =>
            { buffer.push(0u8); buffer.push(70u8); }, ErrorKind ::
            EmptyCharLiteral => { buffer.push(0u8); buffer.push(75u8); },
            ErrorKind :: UnterminatedBlockComment =>
            { buffer.push(0u8); buffer.push(80u8); }, ErrorKind :: InvalidUtf8
            => { buffer.push(0u8); buffer.push(85u8); }, ErrorKind ::
            InvalidUnicodeCharacter =>
            { buffer.push(0u8); buffer.push(90u8); }, ErrorKind ::
            InvalidUnicodeEscape => { buffer.push(0u8); buffer.push(95u8); },
            ErrorKind :: UnmatchedGroup { r#expected, r#got, } =>
            {
                buffer.push(0u8); buffer.push(100u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: TooManyQuotes =>
            { buffer.push(0u8); buffer.push(105u8); }, ErrorKind ::
            UnclosedDelimiter(t0,) =>
            { buffer.push(0u8); buffer.push(110u8); t0.encode_impl(buffer); },
            ErrorKind :: UnexpectedToken { r#expected, r#got, } =>
            {
                buffer.push(0u8); buffer.push(115u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: UnexpectedEof { r#expected, } =>
            {
                buffer.push(0u8); buffer.push(120u8);
                r#expected.encode_impl(buffer);
            }, ErrorKind :: UnexpectedEog { r#expected, } =>
            {
                buffer.push(0u8); buffer.push(125u8);
                r#expected.encode_impl(buffer);
            }, ErrorKind :: MissingDocComment =>
            { buffer.push(0u8); buffer.push(130u8); }, ErrorKind ::
            DocCommentNotAllowed => { buffer.push(0u8); buffer.push(135u8); },
            ErrorKind :: DanglingDocComment =>
            { buffer.push(0u8); buffer.push(136u8); }, ErrorKind ::
            ModuleDocCommentNotAtTop =>
            { buffer.push(0u8); buffer.push(140u8); }, ErrorKind ::
            MissingDecorator(t0,) =>
            { buffer.push(0u8); buffer.push(145u8); t0.encode_impl(buffer); },
            ErrorKind :: DecoratorNotAllowed =>
            { buffer.push(0u8); buffer.push(150u8); }, ErrorKind ::
            DanglingDecorator => { buffer.push(0u8); buffer.push(151u8); },
            ErrorKind :: UnexpectedDecorator(t0,) =>
            { buffer.push(0u8); buffer.push(155u8); t0.encode_impl(buffer); },
            ErrorKind :: ModuleDecoratorNotAtTop =>
            { buffer.push(0u8); buffer.push(160u8); }, ErrorKind ::
            MissingVisibility => { buffer.push(0u8); buffer.push(165u8); },
            ErrorKind :: CannotBePublic =>
            { buffer.push(0u8); buffer.push(170u8); }, ErrorKind ::
            DanglingVisibility => { buffer.push(0u8); buffer.push(171u8); },
            ErrorKind :: FunctionWithoutBody =>
            { buffer.push(0u8); buffer.push(175u8); }, ErrorKind ::
            BlockWithoutValue => { buffer.push(0u8); buffer.push(180u8); },
            ErrorKind :: StructWithoutField =>
            { buffer.push(0u8); buffer.push(185u8); }, ErrorKind ::
            EmptyCurlyBraceBlock => { buffer.push(0u8); buffer.push(190u8); },
            ErrorKind :: PositionalArgAfterKeywordArg =>
            { buffer.push(0u8); buffer.push(195u8); }, ErrorKind ::
            NonDefaultValueAfterDefaultValue =>
            { buffer.push(0u8); buffer.push(200u8); }, ErrorKind ::
            CannotDeclareInlineModule =>
            { buffer.push(0u8); buffer.push(205u8); }, ErrorKind ::
            InclusiveRangeWithNoEnd =>
            { buffer.push(0u8); buffer.push(210u8); }, ErrorKind ::
            MultipleRestPatterns => { buffer.push(0u8); buffer.push(215u8); },
            ErrorKind :: DifferentNameBindingsInOrPattern =>
            { buffer.push(0u8); buffer.push(220u8); }, ErrorKind ::
            InvalidFnType => { buffer.push(0u8); buffer.push(225u8); },
            ErrorKind :: EmptyMatchStatement =>
            { buffer.push(0u8); buffer.push(230u8); }, ErrorKind ::
            RedundantDecorator(t0,) =>
            { buffer.push(0u8); buffer.push(235u8); t0.encode_impl(buffer); },
            ErrorKind :: InvalidDecorator(t0,) =>
            { buffer.push(0u8); buffer.push(240u8); t0.encode_impl(buffer); },
            ErrorKind :: MissingDecoratorArgument { r#expected, r#got, } =>
            {
                buffer.push(0u8); buffer.push(245u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: UnexpectedDecoratorArgument { r#expected, r#got, }
            =>
            {
                buffer.push(0u8); buffer.push(250u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: WrongNumberOfLangItemGenerics
            { r#lang_items, r#generic_def, } =>
            {
                buffer.push(0u8); buffer.push(255u8);
                r#lang_items.encode_impl(buffer);
                r#generic_def.encode_impl(buffer);
            }, ErrorKind :: CannotEvaluateConst =>
            { buffer.push(1u8); buffer.push(4u8); }, ErrorKind ::
            InvalidRangePattern => { buffer.push(1u8); buffer.push(9u8); },
            ErrorKind :: InvalidConcatPattern =>
            { buffer.push(1u8); buffer.push(14u8); }, ErrorKind ::
            CannotBindName(t0,) =>
            { buffer.push(1u8); buffer.push(19u8); t0.encode_impl(buffer); },
            ErrorKind :: CannotApplyInfixOpToMultipleBindings =>
            { buffer.push(1u8); buffer.push(24u8); }, ErrorKind ::
            CannotApplyInfixOpToBinding =>
            { buffer.push(1u8); buffer.push(29u8); }, ErrorKind ::
            CannotAnnotateType => { buffer.push(1u8); buffer.push(34u8); },
            ErrorKind :: RedundantNameBinding(t0, t1,) =>
            {
                buffer.push(1u8); buffer.push(39u8); t0.encode_impl(buffer);
                t1.encode_impl(buffer);
            }, ErrorKind :: UnsupportedInfixOpInPattern(t0,) =>
            { buffer.push(1u8); buffer.push(44u8); t0.encode_impl(buffer); },
            ErrorKind :: NameCollision { r#name, r#kind, } =>
            {
                buffer.push(1u8); buffer.push(49u8);
                r#name.encode_impl(buffer); r#kind.encode_impl(buffer);
            }, ErrorKind :: CyclicLet { r#names, } =>
            {
                buffer.push(1u8); buffer.push(54u8);
                r#names.encode_impl(buffer);
            }, ErrorKind :: CyclicAlias { r#names, } =>
            {
                buffer.push(1u8); buffer.push(59u8);
                r#names.encode_impl(buffer);
            }, ErrorKind :: DollarOutsidePipeline =>
            { buffer.push(1u8); buffer.push(64u8); }, ErrorKind ::
            DisconnectedPipeline => { buffer.push(1u8); buffer.push(69u8); },
            ErrorKind :: UndefinedName(t0,) =>
            { buffer.push(1u8); buffer.push(74u8); t0.encode_impl(buffer); },
            ErrorKind :: EnumVariantInTypeAnnotation =>
            { buffer.push(1u8); buffer.push(79u8); }, ErrorKind ::
            KeywordArgumentRepeated(t0,) =>
            { buffer.push(1u8); buffer.push(84u8); t0.encode_impl(buffer); },
            ErrorKind :: KeywordArgumentNotAllowed =>
            { buffer.push(1u8); buffer.push(89u8); }, ErrorKind ::
            AliasResolveRecursionLimitReached =>
            { buffer.push(1u8); buffer.push(94u8); }, ErrorKind ::
            MissingTypeParameter { r#expected, r#got, } =>
            {
                buffer.push(1u8); buffer.push(99u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: UnexpectedTypeParameter { r#expected, r#got, } =>
            {
                buffer.push(1u8); buffer.push(104u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: MissingKeywordArgument(t0,) =>
            { buffer.push(1u8); buffer.push(110u8); t0.encode_impl(buffer); },
            ErrorKind :: InvalidKeywordArgument(t0,) =>
            { buffer.push(1u8); buffer.push(114u8); t0.encode_impl(buffer); },
            ErrorKind :: MissingFunctionParameter { r#expected, r#got, } =>
            {
                buffer.push(1u8); buffer.push(119u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: UnexpectedFunctionParameter { r#expected, r#got, }
            =>
            {
                buffer.push(1u8); buffer.push(124u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: StructFieldRepeated(t0,) =>
            { buffer.push(1u8); buffer.push(129u8); t0.encode_impl(buffer); },
            ErrorKind :: MissingStructFields
            { r#struct_name, r#missing_fields, } =>
            {
                buffer.push(1u8); buffer.push(134u8);
                r#struct_name.encode_impl(buffer);
                r#missing_fields.encode_impl(buffer);
            }, ErrorKind :: InvalidStructFields
            { r#struct_name, r#invalid_fields, } =>
            {
                buffer.push(1u8); buffer.push(139u8);
                r#struct_name.encode_impl(buffer);
                r#invalid_fields.encode_impl(buffer);
            }, ErrorKind :: CannotAssociateItem =>
            { buffer.push(1u8); buffer.push(142u8); }, ErrorKind ::
            TooGeneralToAssociateItem =>
            { buffer.push(1u8); buffer.push(143u8); }, ErrorKind ::
            DependentTypeNotAllowed =>
            { buffer.push(1u8); buffer.push(144u8); }, ErrorKind ::
            NotCallable { r#type, } =>
            {
                buffer.push(1u8); buffer.push(148u8);
                r#type.encode_impl(buffer);
            }, ErrorKind :: NotStruct { r#id, } =>
            {
                buffer.push(1u8); buffer.push(149u8);
                r#id.encode_impl(buffer);
            }, ErrorKind :: NotExpr { r#id, r#kind, } =>
            {
                buffer.push(1u8); buffer.push(150u8);
                r#id.encode_impl(buffer); r#kind.encode_impl(buffer);
            }, ErrorKind :: NotPolyGeneric { r#id, } =>
            {
                buffer.push(1u8); buffer.push(154u8);
                r#id.encode_impl(buffer);
            }, ErrorKind :: UnexpectedType { r#expected, r#got, } =>
            {
                buffer.push(1u8); buffer.push(159u8);
                r#expected.encode_impl(buffer); r#got.encode_impl(buffer);
            }, ErrorKind :: CannotInferType { r#id, r#is_return, } =>
            {
                buffer.push(1u8); buffer.push(164u8);
                r#id.encode_impl(buffer); r#is_return.encode_impl(buffer);
            }, ErrorKind :: PartiallyInferedType
            { r#id, r#type, r#is_return, } =>
            {
                buffer.push(1u8); buffer.push(169u8);
                r#id.encode_impl(buffer); r#type.encode_impl(buffer);
                r#is_return.encode_impl(buffer);
            }, ErrorKind :: CannotInferGenericType { r#id, } =>
            {
                buffer.push(1u8); buffer.push(174u8);
                r#id.encode_impl(buffer);
            }, ErrorKind :: PartiallyInferedGenericType { r#id, r#type, } =>
            {
                buffer.push(1u8); buffer.push(179u8);
                r#id.encode_impl(buffer); r#type.encode_impl(buffer);
            }, ErrorKind :: CannotApplyInfixOp { r#op, r#arg_types, } =>
            {
                buffer.push(1u8); buffer.push(184u8);
                r#op.encode_impl(buffer); r#arg_types.encode_impl(buffer);
            }, ErrorKind :: CannotSpecializePolyGeneric { r#num_candidates, }
            =>
            {
                buffer.push(1u8); buffer.push(189u8);
                r#num_candidates.encode_impl(buffer);
            }, ErrorKind :: ImpureCallInPureContext =>
            { buffer.push(1u8); buffer.push(194u8); }, ErrorKind ::
            NonExhaustiveArms => { buffer.push(1u8); buffer.push(199u8); },
            ErrorKind :: MultipleModuleFiles { r#module, r#found_files, } =>
            {
                buffer.push(1u8); buffer.push(204u8);
                r#module.encode_impl(buffer);
                r#found_files.encode_impl(buffer);
            }, ErrorKind :: ModuleFileNotFound { r#module, r#candidates, } =>
            {
                buffer.push(1u8); buffer.push(209u8);
                r#module.encode_impl(buffer);
                r#candidates.encode_impl(buffer);
            }, ErrorKind :: LibFileNotFound =>
            { buffer.push(1u8); buffer.push(214u8); }, ErrorKind ::
            SelfParamWithTypeAnnotation =>
            { buffer.push(1u8); buffer.push(219u8); }, ErrorKind ::
            AssociatedFuncWithoutSelfParam =>
            { buffer.push(1u8); buffer.push(224u8); }, ErrorKind ::
            UnusedNames { r#names, r#kind, } =>
            {
                buffer.push(19u8); buffer.push(136u8);
                r#names.encode_impl(buffer); r#kind.encode_impl(buffer);
            }, ErrorKind :: UnreachableMatchArm =>
            { buffer.push(19u8); buffer.push(141u8); }, ErrorKind ::
            NoImpureCallInImpureContext =>
            { buffer.push(19u8); buffer.push(146u8); }, ErrorKind ::
            FuncWithoutTypeAnnotation =>
            { buffer.push(31u8); buffer.push(64u8); }, ErrorKind ::
            LetWithoutTypeAnnotation =>
            { buffer.push(31u8); buffer.push(69u8); }, ErrorKind ::
            FieldWithoutTypeAnnotation =>
            { buffer.push(31u8); buffer.push(74u8); }, ErrorKind ::
            SelfParamNotNamedSelf =>
            { buffer.push(31u8); buffer.push(79u8); }, ErrorKind :: Todo
            { r#id, r#message, } =>
            {
                buffer.push(39u8); buffer.push(14u8);
                r#id.encode_impl(buffer); r#message.encode_impl(buffer);
            }, ErrorKind :: InternalCompilerError { r#id, } =>
            {
                buffer.push(39u8); buffer.push(15u8);
                r#id.encode_impl(buffer);
            },
        }
    } fn decode_impl(buffer : & [u8], mut cursor : usize) -> Result <
    (Self, usize), DecodeError >
    {
        let variant = match (buffer.get(cursor), buffer.get(cursor + 1usize))
        {
            (Some(x), Some(y)) => ((* x as u16) << 8u32) | * y as u16, _ =>
            { return Err(DecodeError :: UnexpectedEof); }
        }; cursor += 2usize; match variant
        {
            0u16 => Ok((ErrorKind :: InvalidNumberLiteral, cursor)), 5u16 =>
            {
                let (t0, cursor) = Vec :: < u8 > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InvalidStringLiteralPrefix(t0,), cursor))
            }, 10u16 => Ok((ErrorKind :: EmptyIdent, cursor)), 15u16 =>
            {
                let (t0, cursor) = char :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InvalidCharacterInIdent(t0,), cursor))
            }, 20u16 =>
            Ok((ErrorKind :: WrongNumberOfQuotesInRawStringLiteral, cursor)),
            25u16 => Ok((ErrorKind :: UnterminatedStringLiteral, cursor)),
            30u16 =>
            {
                let (t0, cursor) = u8 :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: NotAllowedCharInFormattedString(t0,),
                cursor))
            }, 35u16 =>
            Ok((ErrorKind :: UnmatchedBraceInFormattedString, cursor)), 40u16
            => Ok((ErrorKind :: EmptyBraceInFormattedString, cursor)), 45u16
            => Ok((ErrorKind :: DotDotDot, cursor)), 50u16 =>
            Ok((ErrorKind :: InvalidCharLiteral, cursor)), 55u16 =>
            {
                let (t0, cursor) = Vec :: < u8 > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InvalidCharLiteralPrefix(t0,), cursor))
            }, 60u16 => Ok((ErrorKind :: UnterminatedCharLiteral, cursor)),
            65u16 => Ok((ErrorKind :: InvalidByteLiteral, cursor)), 70u16 =>
            Ok((ErrorKind :: InvalidEscape, cursor)), 75u16 =>
            Ok((ErrorKind :: EmptyCharLiteral, cursor)), 80u16 =>
            Ok((ErrorKind :: UnterminatedBlockComment, cursor)), 85u16 =>
            Ok((ErrorKind :: InvalidUtf8, cursor)), 90u16 =>
            Ok((ErrorKind :: InvalidUnicodeCharacter, cursor)), 95u16 =>
            Ok((ErrorKind :: InvalidUnicodeEscape, cursor)), 100u16 =>
            {
                let (r#expected, cursor) = u8 :: decode_impl(buffer, cursor) ?
                ; let (r#got, cursor) = u8 :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnmatchedGroup { r#expected, r#got, },
                cursor))
            }, 105u16 => Ok((ErrorKind :: TooManyQuotes, cursor)), 110u16 =>
            {
                let (t0, cursor) = u8 :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnclosedDelimiter(t0,), cursor))
            }, 115u16 =>
            {
                let (r#expected, cursor) = ErrorToken ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) =
                ErrorToken :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedToken { r#expected, r#got, },
                cursor))
            }, 120u16 =>
            {
                let (r#expected, cursor) = ErrorToken ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedEof { r#expected, }, cursor))
            }, 125u16 =>
            {
                let (r#expected, cursor) = ErrorToken ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedEog { r#expected, }, cursor))
            }, 130u16 => Ok((ErrorKind :: MissingDocComment, cursor)), 135u16
            => Ok((ErrorKind :: DocCommentNotAllowed, cursor)), 136u16 =>
            Ok((ErrorKind :: DanglingDocComment, cursor)), 140u16 =>
            Ok((ErrorKind :: ModuleDocCommentNotAtTop, cursor)), 145u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MissingDecorator(t0,), cursor))
            }, 150u16 => Ok((ErrorKind :: DecoratorNotAllowed, cursor)),
            151u16 => Ok((ErrorKind :: DanglingDecorator, cursor)), 155u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedDecorator(t0,), cursor))
            }, 160u16 => Ok((ErrorKind :: ModuleDecoratorNotAtTop, cursor)),
            165u16 => Ok((ErrorKind :: MissingVisibility, cursor)), 170u16 =>
            Ok((ErrorKind :: CannotBePublic, cursor)), 171u16 =>
            Ok((ErrorKind :: DanglingVisibility, cursor)), 175u16 =>
            Ok((ErrorKind :: FunctionWithoutBody, cursor)), 180u16 =>
            Ok((ErrorKind :: BlockWithoutValue, cursor)), 185u16 =>
            Ok((ErrorKind :: StructWithoutField, cursor)), 190u16 =>
            Ok((ErrorKind :: EmptyCurlyBraceBlock, cursor)), 195u16 =>
            Ok((ErrorKind :: PositionalArgAfterKeywordArg, cursor)), 200u16 =>
            Ok((ErrorKind :: NonDefaultValueAfterDefaultValue, cursor)),
            205u16 => Ok((ErrorKind :: CannotDeclareInlineModule, cursor)),
            210u16 => Ok((ErrorKind :: InclusiveRangeWithNoEnd, cursor)),
            215u16 => Ok((ErrorKind :: MultipleRestPatterns, cursor)), 220u16
            => Ok((ErrorKind :: DifferentNameBindingsInOrPattern, cursor)),
            225u16 => Ok((ErrorKind :: InvalidFnType, cursor)), 230u16 =>
            Ok((ErrorKind :: EmptyMatchStatement, cursor)), 235u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: RedundantDecorator(t0,), cursor))
            }, 240u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InvalidDecorator(t0,), cursor))
            }, 245u16 =>
            {
                let (r#expected, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MissingDecoratorArgument
                { r#expected, r#got, }, cursor))
            }, 250u16 =>
            {
                let (r#expected, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedDecoratorArgument
                { r#expected, r#got, }, cursor))
            }, 255u16 =>
            {
                let (r#lang_items, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#generic_def, cursor) =
                usize :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: WrongNumberOfLangItemGenerics
                { r#lang_items, r#generic_def, }, cursor))
            }, 260u16 => Ok((ErrorKind :: CannotEvaluateConst, cursor)),
            265u16 => Ok((ErrorKind :: InvalidRangePattern, cursor)), 270u16
            => Ok((ErrorKind :: InvalidConcatPattern, cursor)), 275u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CannotBindName(t0,), cursor))
            }, 280u16 =>
            Ok((ErrorKind :: CannotApplyInfixOpToMultipleBindings, cursor)),
            285u16 => Ok((ErrorKind :: CannotApplyInfixOpToBinding, cursor)),
            290u16 => Ok((ErrorKind :: CannotAnnotateType, cursor)), 295u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ; let (t1, cursor) =
                InternedString :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: RedundantNameBinding(t0, t1,), cursor))
            }, 300u16 =>
            {
                let (t0, cursor) = InfixOp :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnsupportedInfixOpInPattern(t0,), cursor))
            }, 305u16 =>
            {
                let (r#name, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ; let (r#kind, cursor) =
                NameCollisionKind :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: NameCollision { r#name, r#kind, }, cursor))
            }, 310u16 =>
            {
                let (r#names, cursor) = Vec :: < InternedString > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CyclicLet { r#names, }, cursor))
            }, 315u16 =>
            {
                let (r#names, cursor) = Vec :: < InternedString > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CyclicAlias { r#names, }, cursor))
            }, 320u16 => Ok((ErrorKind :: DollarOutsidePipeline, cursor)),
            325u16 => Ok((ErrorKind :: DisconnectedPipeline, cursor)), 330u16
            =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UndefinedName(t0,), cursor))
            }, 335u16 =>
            Ok((ErrorKind :: EnumVariantInTypeAnnotation, cursor)), 340u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: KeywordArgumentRepeated(t0,), cursor))
            }, 345u16 => Ok((ErrorKind :: KeywordArgumentNotAllowed, cursor)),
            350u16 =>
            Ok((ErrorKind :: AliasResolveRecursionLimitReached, cursor)),
            355u16 =>
            {
                let (r#expected, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MissingTypeParameter { r#expected, r#got, },
                cursor))
            }, 360u16 =>
            {
                let (r#expected, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedTypeParameter
                { r#expected, r#got, }, cursor))
            }, 366u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MissingKeywordArgument(t0,), cursor))
            }, 370u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InvalidKeywordArgument(t0,), cursor))
            }, 375u16 =>
            {
                let (r#expected, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MissingFunctionParameter
                { r#expected, r#got, }, cursor))
            }, 380u16 =>
            {
                let (r#expected, cursor) = usize ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedFunctionParameter
                { r#expected, r#got, }, cursor))
            }, 385u16 =>
            {
                let (t0, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: StructFieldRepeated(t0,), cursor))
            }, 390u16 =>
            {
                let (r#struct_name, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ; let (r#missing_fields, cursor)
                = Vec :: < InternedString > :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MissingStructFields
                { r#struct_name, r#missing_fields, }, cursor))
            }, 395u16 =>
            {
                let (r#struct_name, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ; let (r#invalid_fields, cursor)
                = Vec :: < InternedString > :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InvalidStructFields
                { r#struct_name, r#invalid_fields, }, cursor))
            }, 398u16 => Ok((ErrorKind :: CannotAssociateItem, cursor)),
            399u16 => Ok((ErrorKind :: TooGeneralToAssociateItem, cursor)),
            400u16 => Ok((ErrorKind :: DependentTypeNotAllowed, cursor)),
            404u16 =>
            {
                let (r#type, cursor) = String :: decode_impl(buffer, cursor) ?
                ; Ok((ErrorKind :: NotCallable { r#type, }, cursor))
            }, 405u16 =>
            {
                let (r#id, cursor) = Option :: < IdentWithOrigin > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: NotStruct { r#id, }, cursor))
            }, 406u16 =>
            {
                let (r#id, cursor) = InternedString ::
                decode_impl(buffer, cursor) ? ; let (r#kind, cursor) =
                NotExprBut :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: NotExpr { r#id, r#kind, }, cursor))
            }, 410u16 =>
            {
                let (r#id, cursor) = Option :: < IdentWithOrigin > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: NotPolyGeneric { r#id, }, cursor))
            }, 415u16 =>
            {
                let (r#expected, cursor) = String ::
                decode_impl(buffer, cursor) ? ; let (r#got, cursor) = String
                :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnexpectedType { r#expected, r#got, },
                cursor))
            }, 420u16 =>
            {
                let (r#id, cursor) = Option :: < InternedString >::
                decode_impl(buffer, cursor) ? ; let (r#is_return, cursor) =
                bool :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CannotInferType { r#id, r#is_return, },
                cursor))
            }, 425u16 =>
            {
                let (r#id, cursor) = Option :: < InternedString >::
                decode_impl(buffer, cursor) ? ; let (r#type, cursor) = String
                :: decode_impl(buffer, cursor) ? ; let (r#is_return, cursor) =
                bool :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: PartiallyInferedType
                { r#id, r#type, r#is_return, }, cursor))
            }, 430u16 =>
            {
                let (r#id, cursor) = Option :: < String > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CannotInferGenericType { r#id, }, cursor))
            }, 435u16 =>
            {
                let (r#id, cursor) = Option :: < String >::
                decode_impl(buffer, cursor) ? ; let (r#type, cursor) = String
                :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: PartiallyInferedGenericType
                { r#id, r#type, }, cursor))
            }, 440u16 =>
            {
                let (r#op, cursor) = InfixOp :: decode_impl(buffer, cursor) ?
                ; let (r#arg_types, cursor) = Vec :: < String > ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CannotApplyInfixOp { r#op, r#arg_types, },
                cursor))
            }, 445u16 =>
            {
                let (r#num_candidates, cursor) = usize ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: CannotSpecializePolyGeneric
                { r#num_candidates, }, cursor))
            }, 450u16 => Ok((ErrorKind :: ImpureCallInPureContext, cursor)),
            455u16 => Ok((ErrorKind :: NonExhaustiveArms, cursor)), 460u16 =>
            {
                let (r#module, cursor) = ModulePath ::
                decode_impl(buffer, cursor) ? ; let (r#found_files, cursor) =
                Vec :: < String > :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: MultipleModuleFiles
                { r#module, r#found_files, }, cursor))
            }, 465u16 =>
            {
                let (r#module, cursor) = ModulePath ::
                decode_impl(buffer, cursor) ? ; let (r#candidates, cursor) =
                Vec :: < String > :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: ModuleFileNotFound
                { r#module, r#candidates, }, cursor))
            }, 470u16 => Ok((ErrorKind :: LibFileNotFound, cursor)), 475u16 =>
            Ok((ErrorKind :: SelfParamWithTypeAnnotation, cursor)), 480u16 =>
            Ok((ErrorKind :: AssociatedFuncWithoutSelfParam, cursor)), 5000u16
            =>
            {
                let (r#names, cursor) = Vec :: < InternedString >::
                decode_impl(buffer, cursor) ? ; let (r#kind, cursor) =
                NameKind :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: UnusedNames { r#names, r#kind, }, cursor))
            }, 5005u16 => Ok((ErrorKind :: UnreachableMatchArm, cursor)),
            5010u16 => Ok((ErrorKind :: NoImpureCallInImpureContext, cursor)),
            8000u16 => Ok((ErrorKind :: FuncWithoutTypeAnnotation, cursor)),
            8005u16 => Ok((ErrorKind :: LetWithoutTypeAnnotation, cursor)),
            8010u16 => Ok((ErrorKind :: FieldWithoutTypeAnnotation, cursor)),
            8015u16 => Ok((ErrorKind :: SelfParamNotNamedSelf, cursor)),
            9998u16 =>
            {
                let (r#id, cursor) = u32 :: decode_impl(buffer, cursor) ? ;
                let (r#message, cursor) = String ::
                decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: Todo { r#id, r#message, }, cursor))
            }, 9999u16 =>
            {
                let (r#id, cursor) = u32 :: decode_impl(buffer, cursor) ? ;
                Ok((ErrorKind :: InternalCompilerError { r#id, }, cursor))
            }, _ =>
            Err(DecodeError :: InvalidLargeEnumVariant(variant as u32)),
        }
    }
} 