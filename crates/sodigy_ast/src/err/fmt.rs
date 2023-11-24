use super::ExpectedToken;
use sodigy_err::concat_commas;
use std::fmt;

impl fmt::Display for ExpectedToken {
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
                ExpectedToken::IdentOrBrace => "an identifier or `{}`".to_string(),
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
