#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delim {
    Parenthesis,
    Bracket,
    Brace,
    Lambda,  // \()
    Decorator,  // #[]
    ModuleDecorator,  // #![]
}
