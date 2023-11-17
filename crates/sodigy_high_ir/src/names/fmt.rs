use super::NameBindingType;

impl NameBindingType {
    // "unused {self}" should make sense
    pub fn render_error(&self) -> String {
        match self {
            NameBindingType::LocalScope => "local name binding",
            NameBindingType::FuncArg => "function argument",
            NameBindingType::FuncGeneric => "generic",
            NameBindingType::LambdaArg => "lambda argument",
            NameBindingType::MatchArm => "name binding in match arm",
            NameBindingType::IfLet => "name binding in an `if let` clause",
            NameBindingType::Import => "import",
        }.to_string()
    }
}
