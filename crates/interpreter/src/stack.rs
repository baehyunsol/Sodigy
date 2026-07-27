use sodigy_bytecode::SSA;
use std::collections::HashMap;

pub struct Stack {
    // TODO: It's toooooo inefficient to implement ssa registers this way.
    pub ssa: HashMap<SSA, u32>,
    pub r#return: u32,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            ssa: HashMap::new(),
            r#return: 0,
        }
    }

    pub fn from_args(args: &[SSA], old_stack: &Stack) -> Stack {
        Stack {
            ssa: args.iter().enumerate().map(
                |(i, arg)| (SSA::from_u32(i as u32), *old_stack.ssa.get(arg).unwrap())
            ).collect(),
            r#return: 0,
        }
    }
}
