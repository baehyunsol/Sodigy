use crate::Value;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Stack {
    pub func_args: Vec<Vec<Value>>,
    pub block: Vec<HashMap<Span, Value>>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            func_args: vec![],
            block: vec![],
        }
    }
}
