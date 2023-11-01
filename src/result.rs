pub struct CompileResult {
    // TODO: I want Vec<Box<dyn SodigyError>>
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl CompileResult {
    pub fn new() -> Self {
        CompileResult {
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn push_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn push_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn concat_errors(&self) -> String {
        self.errors.join("\n\n")
    }

    pub fn concat_warnings(&self) -> String {
        self.warnings.join("\n\n")
    }
}
