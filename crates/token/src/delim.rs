#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Delim {
    Parenthesis,
    Bracket,
    Brace,
    Lambda,  // \()
    Decorator,  // #[]
    ModuleDecorator,  // #![]
}
