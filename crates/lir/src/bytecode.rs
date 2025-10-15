use crate::{Const, Label, Register};
use sodigy_mir::Intrinsic;

#[derive(Clone, Copy, Debug)]
pub enum Bytecode {
    // It peeks a value from `src` and pushes that to `dst`.
    Push {
        src: Register,
        dst: Register,
    },
    PushConst {
        value: Const,
        dst: Register,
    },
    Pop(Register),
    PushCallStack(Label),
    PopCallStack,
    Goto(Label),

    // It's like a function call, but is always inlined.
    // For example, if it's `Intrinsic::IntegerAdd`, it adds
    // `Register::Call(0)` and `Register::Call(1)` and stores
    // the result at `Register::Return`.
    Intrinsic(Intrinsic),

    // After calling `into_labeled_bytecode`, `Bytecode::Label` must all be gone.
    Label(Label),

    // goto(call_stack.peek());
    // It doesn't pop `call_stack`.
    Return,

    // It's guaranteed that `value` has type `Bool`.
    JumpIf {
        value: Register,
        label: Label,
    },

    // It's used for lazy-eval values.
    JumpIfInit {
        reg: Register,
        label: Label,
    },
}

impl Bytecode {
    pub fn is_unconditional_jump(&self) -> bool {
        match self {
            Bytecode::Goto(_) |
            Bytecode::Return => true,
            Bytecode::Push { .. } |
            Bytecode::PushConst { .. } |
            Bytecode::Pop(_) |
            Bytecode::PopCallStack |
            Bytecode::PushCallStack(_) |
            Bytecode::Label(_) |
            Bytecode::JumpIf { .. } |
            Bytecode::JumpIfInit { .. } => false,
            Bytecode::Intrinsic(intrinsic) => match intrinsic {
                Intrinsic::Panic |
                Intrinsic::Exit => true,
                Intrinsic::IntegerAdd |
                Intrinsic::IntegerSub |
                Intrinsic::IntegerMul |
                Intrinsic::IntegerDiv |
                Intrinsic::IntegerEq |
                Intrinsic::IntegerGt |
                Intrinsic::IntegerLt |
                Intrinsic::Print |
                Intrinsic::EPrint => false,
            },
        }
    }
}
