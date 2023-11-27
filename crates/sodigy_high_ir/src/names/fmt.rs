use super::NameBindingType;

impl NameBindingType {
    // "unused {self}" should make sense
    pub fn render_error(&self) -> String {
        match self {
            NameBindingType::ScopedLet => "local name binding in a scoped let",
            NameBindingType::FuncArg => "function argument",
            NameBindingType::FuncGeneric => "generic",
            NameBindingType::LambdaArg => "lambda argument",
            NameBindingType::MatchArm => "name binding in match arm",
            NameBindingType::IfPattern => "name binding in an `if pattern` clause",
            NameBindingType::Import => "import",
        }.to_string()
    }
}
