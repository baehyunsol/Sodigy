#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    Let,
    Func,
    Struct,
    Enum,
    Module,
    Use,
    If,
    Else,
    Pat,
    Match,
}
