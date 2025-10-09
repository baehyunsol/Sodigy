#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    Let,
    Fn,
    Struct,
    Enum,
    Module,
    Use,
    If,
    Else,
    Match,
}
