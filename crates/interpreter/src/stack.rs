use std::collections::HashMap;

pub struct Stack {
    // TODO: It's toooooo inefficient to implement ssa registers this way.
    pub ssa: HashMap<u32, u32>,
    pub r#return: u32,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            ssa: HashMap::new(),
            r#return: 0,
        }
    }

    pub fn from_args(args: &[u32], old_stack: &Stack) -> Stack {
        Stack {
            ssa: args.iter().enumerate().map(
                |(i, arg)| (i as u32, *old_stack.ssa.get(arg).unwrap())
            ).collect(),
            r#return: 0,
        }
    }
}
