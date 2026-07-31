#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Delim {
    Parenthesis,
    Bracket,
    Brace,
    Lambda,  // \()
    Decorator,  // #[]
    ModuleDecorator,  // #![]
}

impl Delim {
    pub fn open(&self) -> &'static str {
        match self {
            Delim::Parenthesis => "(",
            Delim::Bracket => "[",
            Delim::Brace => "{",
            Delim::Lambda => "\\(",
            Delim::Decorator => "#[",
            Delim::ModuleDecorator => "#![",
        }
    }

    pub fn close(&self) -> &'static str {
        match self {
            Delim::Parenthesis => ")",
            Delim::Bracket => "]",
            Delim::Brace => "}",
            Delim::Lambda => ")",
            Delim::Decorator => "]",
            Delim::ModuleDecorator => "]",
        }
    }

    pub fn markers(&self) -> (&'static str, &'static str) {
        (self.open(), self.close())
    }
}
