use crate::Value;

pub enum Token {
    Assign,
    Semicolon,
    InternedValue(u128),
    Value(Value),
}
