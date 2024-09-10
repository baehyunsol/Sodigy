mod endec;

// TODO: This enum is too ambiguous
//       1. some have contexts, while some do not. and there's no clear reason
//       2. what if an expr is in a match body in a lambda body in a func body? which context should it have?
//       3. it adds an extra info to error messages, but that does not help the users at all
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
    ParsingTypeAnnotation,
    ParsingTypeInPattern,
}
