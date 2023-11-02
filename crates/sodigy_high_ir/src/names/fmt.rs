use super::NameBindingType;
use std::fmt;

impl fmt::Display for NameBindingType {
    // "unused {self}" should make sense
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            NameBindingType::LocalScope => "local name binding",
            NameBindingType::FuncArg => "function argument",
            NameBindingType::FuncGeneric => "generic",
            NameBindingType::LambdaArg => "lambda argument",
            NameBindingType::MatchArm => "name binding in match arm",
            NameBindingType::IfLet => "name binding in an `if let` clause",
        };

        write!(fmt, "{s}")
    }
}