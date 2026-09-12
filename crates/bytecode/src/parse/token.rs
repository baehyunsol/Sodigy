use crate::Value;

pub enum Token {
    Assign,
    Semicolon,
    InternedValue(u128),
    Value(Value),
    Decorator {
        ident: Vec<u8>,
        args: Vec<Vec<u8>>,
    },
}
