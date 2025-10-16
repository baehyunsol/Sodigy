use crate::{
    Const,
    ConstOrRegister,
    InPlaceOrRegister,
    Label,
    Offset,
    Register,
};
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

    UpdateCompound {
        // the compound value
        src: Register,

        // an integer (nth element of the compound value)
        offset: Offset,

        // it'll be a new member of the compound
        value: ConstOrRegister,

        // where to store the updated compound value
        dst: InPlaceOrRegister,
    },
    ReadCompound {
        // the compound value
        src: Register,

        // an integer (nth element of the compound value)
        offset: Offset,

        // where to store the read value
        dst: Register,
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
            Bytecode::JumpIfInit { .. } |
            Bytecode::UpdateCompound { .. } |
            Bytecode::ReadCompound { .. } => false,
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
