pub struct Stack {
    pub stack: Vec<u32>,
    pub stack_pointer: usize,
    pub r#return: u32,
    pub call_stack: Vec<usize>,
}

impl Stack {
    pub fn with_capacity(n: usize) -> Stack {
        Stack {
            stack: vec![0; n],
            stack_pointer: 0,
            r#return: 0,
            call_stack: vec![],
        }
    }
}
