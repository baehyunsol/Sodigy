use super::NameBindingType;
use sodigy_error::RenderError;
use std::fmt;

impl NameBindingType {
    // 'a' or 'an' before `self.render_error()`
    pub fn article(&self, capital: bool) -> &'static str {
        match self {
            NameBindingType::ScopedLet
            | NameBindingType::FuncArg
            | NameBindingType::FuncGeneric
            | NameBindingType::LambdaArg
            | NameBindingType::MatchArm
            | NameBindingType::IfPattern => if capital { "A" } else { "a" },
            NameBindingType::Import => if capital { "An" } else { "an" },
        }
    }
}

impl RenderError for NameBindingType {
    // "an unused {self}" should make sense
    fn render_error(&self) -> String {
        match self {
            NameBindingType::ScopedLet => "local name binding in a scoped let",
            NameBindingType::FuncArg => "function argument",
            NameBindingType::FuncGeneric => "generic",
            NameBindingType::LambdaArg => "lambda argument",
            NameBindingType::MatchArm => "name binding in a match arm",
            NameBindingType::IfPattern => "name binding in an `if pattern` clause",
            NameBindingType::Import => "import",
        }.to_string()
    }
}

impl fmt::Display for NameBindingType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
