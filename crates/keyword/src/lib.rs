#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    Let,
    Fn,
    Struct,
    Enum,
    Assert,
    Module,
    Use,
    If,
    Else,
    Match,
}
