mod endec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorContext {
    Unknown,
    ParsingCommandLine,
    ParsingConfigFile,
    ExpandingMacro,
    Lexing,
    LexingNumericLiteral,
    ParsingLetStatement,
    ParsingImportStatement,
    ParsingFuncName,
    ParsingFuncRetType,
    ParsingFuncBody,
    ParsingFuncArgs,
    ParsingEnumBody,
    ParsingStructBody,
    ParsingStructInit,
    ParsingMatchBody,
    ParsingBranchCondition,
    ParsingLambdaBody,
    ParsingScopeBlock,
    ParsingFormattedString,
    ParsingPattern,
    ParsingTypeInPattern,
}
