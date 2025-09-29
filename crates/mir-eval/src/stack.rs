use crate::Value;

pub struct Stack {
    pub func_args: Vec<Vec<Value>>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            func_args: vec![],
        }
    }
}
