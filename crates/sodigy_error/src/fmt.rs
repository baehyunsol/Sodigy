use crate::{
    ErrorContext,
    ExpectedToken,
    concat_commas,
    trim_long_string,
};
use sodigy_endec::EndecErrorContext;
use sodigy_files::FileErrorContext;
use sodigy_intern::{InternedString, unintern_string};
use std::fmt;

/// All the error messages use this function to print objects
pub trait RenderError {
    fn render_error(&self) -> String;
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ErrorContext::Unknown => "",
            ErrorContext::ParsingCommandLine => "parsing command line arguments",
            ErrorContext::ParsingConfigFile => "parsing a configuration file",
            ErrorContext::ExpandingMacro => "expanding a macro",
            ErrorContext::Lexing => "lexing",
            ErrorContext::LexingNumericLiteral => "lexing a numeric literal",
            ErrorContext::ParsingLetStatement => "parsing a let statement",
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

impl RenderError for ErrorContext {
    fn render_error(&self) -> String {
        self.to_string()
    }
}

impl<T: RenderError> fmt::Display for ExpectedToken<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                ExpectedToken::AnyExpression => "an expression".to_string(),
                ExpectedToken::AnyIdentifier => "an identifier".to_string(),
                ExpectedToken::AnyStatement => "a statement".to_string(),
                ExpectedToken::AnyPattern => "a pattern".to_string(),
                ExpectedToken::AnyNumber => "a number".to_string(),
                ExpectedToken::AnyDocComment => "a doc-comment".to_string(),
                ExpectedToken::AnyType => "type".to_string(),
                ExpectedToken::IdentOrBrace => "an identifier or `{...}`".to_string(),
                ExpectedToken::LetStatement => "an identifier or a keyword `enum`, `struct` or `pattern`".to_string(),
                ExpectedToken::Nothing => "nothing".to_string(),
                ExpectedToken::PostExpr => "a postfix operator or an infix operator".to_string(),
                ExpectedToken::FuncArgs => "arguments".to_string(),
                ExpectedToken::Specific(tokens) => concat_commas(
                    &tokens.iter().map(|t| t.render_error()).collect::<Vec<String>>(),
                    "or",
                    "`",
                    "`",
                ),
            },
        )
    }
}

impl RenderError for FileErrorContext {
    fn render_error(&self) -> String {
        match self {
            FileErrorContext::None => "doing file IO",
            FileErrorContext::SavingIr => "saving intermediate representation",
            FileErrorContext::CleaningIr => "cleaning intermediate representations",
            FileErrorContext::DumpingTokensToFile => "dumping tokens to file",
            FileErrorContext::DumpingHirToFile => "dumping hir to file",
            FileErrorContext::DumpingMirToFile => "dumping mir to file",
        }.to_string()
    }
}

impl RenderError for EndecErrorContext {
    fn render_error(&self) -> String {
        match self {
            EndecErrorContext::None => "decoding a file",
            EndecErrorContext::ConstructingTokensFromIr => "constructing tokens from saved ir",
            EndecErrorContext::ConstructingHirFromIr => "constructing hir from saved ir",
        }.to_string()
    }
}

impl RenderError for InternedString {
    fn render_error(&self) -> String {
        let v = unintern_string(*self);
        trim_long_string(String::from_utf8_lossy(&v).to_string(), 16, 16)
    }
}
