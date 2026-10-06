use crate::{GlobalLabel, LocalLabel, Memory, Value};

pub enum Token {
    Assign,
    Semicolon,
    Keyword(Keyword),
    InternedValue(u128),
    Value(Value),
    Decorator {
        ident: Vec<u8>,
        args: Vec<Vec<u8>>,
    },
    LocalLabel(LocalLabel),
    GlobalLabel(GlobalLabel),
    Memory(Memory),
}

pub enum Keyword {
    Call,
    Code,
    If,
    Jump,
}
