use super::ErrorContext;
use std::fmt;

impl fmt::Display for ErrorContext {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ErrorContext::Unknown => "",
            ErrorContext::Lexing => "lexing",
            ErrorContext::LexingNumericLiteral => "lexing a numeric literal",
            ErrorContext::ParsingImportStatement => "parsing an import statement",
            ErrorContext::ParsingFuncBody => "parsing a function body",
            ErrorContext::ParsingFuncName => "parsing name of a function",
            ErrorContext::ParsingFuncRetType => "parsing return type of a function",
            ErrorContext::ParsingFuncArgs => "parsing function arguments",
            ErrorContext::ParsingEnumBody => "parsing an enum body",
            ErrorContext::ParsingStructBody => "parsing a struct body",
            ErrorContext::ParsingStructInit => "parsing a struct initialization",
            ErrorContext::ParsingMatchBody => "parsing a body of a match expression",
            ErrorContext::ParsingBranchCondition => "parsing a condition of a branch",
            ErrorContext::ParsingLambdaBody => "parsing a body of a lambda function",
            ErrorContext::ParsingScopeBlock => "parsing a scope block",
            ErrorContext::ParsingFormattedString => "parsing a formatted string",
            ErrorContext::ParsingPattern => "parsing a pattern",
            ErrorContext::ParsingTypeInPattern => "parsing type of a pattern",
        };

        write!(fmt, "{s}")
    }
}
