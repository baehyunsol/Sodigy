use sodigy_mir::{Intrinsic, Session as MirSession};
use sodigy_span::Span;
use std::collections::HashMap;

mod assert;
mod dump;
mod endec;
mod executable;
mod expr;
mod func;
mod r#let;
mod link;
mod session;
mod value;

#[cfg(test)]
mod tests;

pub use assert::Assert;
pub(crate) use dump::dump_bytecodes;
pub use executable::Executable;
pub(crate) use expr::lower_expr;
pub use func::Func;
pub use r#let::Let;
pub use session::{LocalValue, Session};
pub use value::Value;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SSA(u32);

impl SSA {
    pub fn from_u32(n: u32) -> SSA {
        SSA(n)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

// `debug_info` fields are set only if the session's `debug_info` field is set.

#[derive(Clone, Debug)]
pub enum Bytecode {
    Const {
        value: Value,
        dst: Memory,
        debug_info: Option<Box<Span>>,
    },
    Move {
        src: Memory,
        dst: Memory,
    },
    Phi {
        pair: (SSA, SSA),
        dst: Memory,
    },

    Jump(Label),

    Call {
        func: Label,
        args: Vec<SSA>,

        // The returned value is stored here.
        // It's None iff it's a tail-call.
        dst: Option<Memory>,

        debug_info: Option<Box<Span>>,
    },

    CallDynamic {
        func: Memory,    // function pointer
        args: Vec<SSA>,

        // The returned value is stored here.
        // It's None iff it's a tail-call.
        dst: Option<Memory>,

        debug_info: Option<Box<Span>>,
    },

    // Jumps if the `value` is 1.
    JumpIf {
        value: Memory,
        label: Label,
        debug_info: Option<Box<Span>>,
    },

    // If the global value `def_span` is not initialized, it calls the function `func`.
    // Otherwise, it jumps to `label`.
    InitOrJump {
        def_span: Span,
        func: Label,
        label: Label,
    },

    // Definition of a label.
    Label(Label),

    Return(SSA),

    Update {
        // pointer to a tuple
        src: SSA,

        // of the tuple, so that we know how many elements fo clone
        size: usize,

        // of the element to update
        index: usize,

        value: SSA,
        dst: Memory,
    },

    Intrinsic {
        intrinsic: Intrinsic,
        args: Vec<SSA>,

        // The result of the intrinsic, if exists, will be stored here.
        dst: Memory,
        debug_info: Option<Box<Span>>,
    },

    // `InitTuple` and `InitList` are very similar.
    // It allocates a heap memory and saves the pointer to `dst`.
    // In runtime's point of view, tuples and structs are the same.
    // So the compiler emits `InitTuple` to initialize a struct.
    InitTuple {
        elements: usize,
        dst: Memory,
        debug_info: Option<Box<Span>>,
    },
    InitList {
        elements: usize,
        dst: Memory,
        debug_info: Option<Box<Span>>,
    },

    // The runtime has to implement a special control flow for assertions.
    // An assertion may panic, but there's no (and will never be a) way to
    // catch a panic and recover. Then how does the runtime throw an appropriate
    // error message when an assertion fails?
    //
    // 1. The runtime evaluates the name of the assertion -> it never panics.
    // 2. It pushes the name to DebugInfoStack.
    // 3. If the assertion has a `note`,
    //   3-1. The runtime pushes the span of the note to the stack.
    //   3-2. The runtime evaluates the note -> it may panic.
    //   3-3. It pushes the note to the stack.
    // 4. It evaluates the assertion value -> it may panic.
    // 5. It pops the values in the stack.
    //
    // If step 3-2 fails, there must be a span of the note in the stack, so the
    // runtime knows that something went wrong while evaluating the note, and it
    // generates an error message using values in the stack.
    // Same for the step 4.
    //
    // It moves the data, not copying it.
    PushDebugInfo {
        kind: DebugInfoKind,
        src: Memory,
    },
    PopDebugInfo,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Memory {
    // A register for a return value.
    // A return value maybe stored here or directly stored in a SSA register.
    Return,
    SSA(SSA),

    Heap {
        ptr: SSA,
        offset: Offset,
    },

    // `Memory::Heap` and `Memory::List` may or may not be identical.
    // It just gives more hints to the runtime so that the runtime can
    // do optimizations for lists.
    List {
        ptr: SSA,
        offset: Offset,
    },

    // Top-level `let` statements.
    Global(Span),
}

impl Memory {
    pub fn get_heap_index(&self) -> Option<(SSA, u32)> {
        match self {
            Memory::Heap { ptr: a, offset: Offset::Static(b) } |
            Memory::List { ptr: a, offset: Offset::Static(b) } => Some((*a, *b)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Label {
    Local(u32),
    Global(Span /* def_span of the item */),

    // Labels are flattened by `Session::into_exeutable`.
    // After flattened, every label in the executable has a unique id.
    Flatten(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Offset {
    Static(u32),
    Dynamic(Box<Memory>),
}

// TODO: it should be in mir... right?
#[derive(Clone, Debug)]
pub enum DropType {
    // Byte, Char
    // No need for drop
    Scalar,

    // Int, (Byte, Byte)
    // Just decrement its rc.
    SimpleCompound,

    // List is very special because it
    //   1. has an arbitrary number of args
    //   2. has an integer for length
    // So, it has to drop the integer (which is SimpleCompound),
    // and the elements with the given DropType.
    List(Box<DropType>),

    // (Byte, [Char]), (Int, Int)
    Compound(Vec<DropType>),
}

#[derive(Clone, Copy, Debug)]
pub enum DebugInfoKind {
    AssertionKeywordSpan,
    AssertionName,
    AssertionNoteDecoratorSpan,
    AssertionNote,
}

impl Bytecode {
    pub fn get_dst(&self) -> Option<&Memory> {
        match self {
            Bytecode::Const { dst, .. } |
            Bytecode::Move { dst, .. } |
            Bytecode::Phi { dst, .. } |
            Bytecode::Update { dst, .. } |
            Bytecode::Intrinsic { dst, .. } |
            Bytecode::InitTuple { dst, .. } |
            Bytecode::InitList { dst, .. } => Some(dst),
            Bytecode::Call { dst, .. } |
            Bytecode::CallDynamic { dst, .. } => dst.as_ref(),
            Bytecode::Jump(_) |
            Bytecode::JumpIf { .. } |
            Bytecode::InitOrJump { .. } |
            Bytecode::Label(_) |
            Bytecode::Return(_) |
            Bytecode::PushDebugInfo { .. } |
            Bytecode::PopDebugInfo => None,
        }
    }

    pub fn set_dst(&mut self, new_dst: Memory) {
        match self {
            Bytecode::Const { dst, .. } |
            Bytecode::Move { dst, .. } |
            Bytecode::Phi { dst, .. } |
            Bytecode::Intrinsic { dst, .. } |
            Bytecode::InitTuple { dst, .. } |
            Bytecode::InitList { dst, .. } |
            Bytecode::Call { dst: Some(dst), .. } |
            Bytecode::CallDynamic { dst: Some(dst), .. } => {
                *dst = new_dst;
            },
            _ => panic!("Bytecode {self:?} has no dst."),
        }
    }

    pub fn apply_ssa_alias(&mut self, ssa_alias: &HashMap<SSA, SSA>, heap_ssa_alias: &HashMap<(SSA, u32), SSA>) {
        fn apply_ssa_alias(src: &mut Memory, ssa_alias: &HashMap<SSA, SSA>, heap_ssa_alias: &HashMap<(SSA, u32), SSA>) {
            match src {
                Memory::Return => {},
                Memory::SSA(i) => {
                    *i = *ssa_alias.get(i).unwrap_or(i);
                },
                Memory::Heap { ptr: a, offset: Offset::Static(b) } => {
                    if let Some(c) = heap_ssa_alias.get(&(*a, *b)) {
                        *src = Memory::SSA(*ssa_alias.get(c).unwrap_or(c));
                    }

                    else {
                        *a = *ssa_alias.get(a).unwrap_or(a);
                    }
                },
                Memory::Heap { ptr, .. } => {
                    *ptr = *ssa_alias.get(ptr).unwrap_or(ptr);
                },
                Memory::List { ptr: a, offset: Offset::Static(b) } => {
                    if let Some(c) = heap_ssa_alias.get(&(*a, *b)) {
                        *src = Memory::SSA(*ssa_alias.get(c).unwrap_or(c));
                    }

                    else {
                        *a = *ssa_alias.get(a).unwrap_or(a);
                    }
                },
                Memory::List { ptr, .. } => {
                    *ptr = *ssa_alias.get(ptr).unwrap_or(ptr);
                },
                Memory::Global(_) => {},
            }
        }

        fn apply_ssa_alias_args(args: &mut Vec<SSA>, ssa_alias: &HashMap<SSA, SSA>, heap_ssa_alias: &HashMap<(SSA, u32), SSA>) {
            *args = args.iter().map(|i| *ssa_alias.get(i).unwrap_or(i)).collect();
        }

        // TODO: isn't it supposed to update all the `dst`s?
        match self {
            Bytecode::Const { .. } => {},
            Bytecode::Move { src, dst } => {
                if let Memory::SSA(_) = dst {
                    apply_ssa_alias(dst, ssa_alias, heap_ssa_alias);
                }

                apply_ssa_alias(src, ssa_alias, heap_ssa_alias);
            },
            Bytecode::Phi { pair, .. } => {
                let (mut a, mut b) = *pair;
                a = *ssa_alias.get(&a).unwrap_or(&a);
                b = *ssa_alias.get(&b).unwrap_or(&b);
                *pair = (a, b);
            },
            Bytecode::Jump(_) => {},
            Bytecode::Call { args, .. } => {
                apply_ssa_alias_args(args, ssa_alias, heap_ssa_alias);
            },
            Bytecode::CallDynamic { func, args, .. } => {
                apply_ssa_alias(func, ssa_alias, heap_ssa_alias);
                apply_ssa_alias_args(args, ssa_alias, heap_ssa_alias);
            },
            Bytecode::JumpIf { value, .. } => {
                apply_ssa_alias(value, ssa_alias, heap_ssa_alias);
            },
            Bytecode::InitOrJump { .. } => {},
            Bytecode::Label(_) => {},
            Bytecode::Return(a) => {
                *a = *ssa_alias.get(a).unwrap_or(a);
            },
            Bytecode::Update { src, value, .. } => todo!(),
            Bytecode::Intrinsic { args, .. } => {
                apply_ssa_alias_args(args, ssa_alias, heap_ssa_alias);
            },
            Bytecode::InitTuple { .. } => {},
            Bytecode::InitList { .. } => {},
            Bytecode::PushDebugInfo { src, .. } => {
                apply_ssa_alias(src, ssa_alias, heap_ssa_alias);
            },
            Bytecode::PopDebugInfo => {},
        }
    }

    pub fn debug_info(&self) -> Option<Box<Span>> {
        match self {
            Bytecode::Const { debug_info, .. } |
            Bytecode::Call { debug_info, .. } |
            Bytecode::CallDynamic { debug_info, .. } |
            Bytecode::JumpIf { debug_info, .. } |
            Bytecode::Intrinsic { debug_info, .. } |
            Bytecode::InitTuple { debug_info, .. } |
            Bytecode::InitList { debug_info, .. } => debug_info.clone(),
            _ => None,
        }
    }

    pub fn used_ssa_indexes(&self) -> Vec<SSA> {
        let mut indexes: Vec<SSA> = vec![];
        let mut memories: Vec<Memory> = vec![];

        match self {
            Bytecode::Const { dst: memory, .. } |
            Bytecode::JumpIf { value: memory, .. } |
            Bytecode::InitTuple { dst: memory, .. } |
            Bytecode::InitList { dst: memory, .. } |
            Bytecode::PushDebugInfo { src: memory, .. } => {
                memories.push(memory.clone());
            },
            Bytecode::Move { src, dst } => {
                memories.push(src.clone());
                memories.push(dst.clone());
            },
            Bytecode::Phi { pair: (a, b), dst } => {
                indexes.push(*a);
                indexes.push(*b);
                memories.push(dst.clone());
            },
            Bytecode::Call { args, .. } |
            Bytecode::CallDynamic { args, .. } => {
                indexes.extend(args.to_vec());
            },
            Bytecode::Return(n) => {
                indexes.push(*n);
            },
            Bytecode::Update { src, value, dst, .. } => {
                indexes.push(*src);
                indexes.push(*value);
                memories.push(dst.clone());
            },
            Bytecode::Intrinsic { args, dst, .. } => {
                indexes.extend(args.to_vec());
                memories.push(dst.clone());
            },
            Bytecode::Jump(_) |
            Bytecode::InitOrJump { .. } |
            Bytecode::Label(_) |
            Bytecode::PopDebugInfo => {},
        }

        while let Some(m) = memories.pop() {
            match m {
                Memory::SSA(n) => {
                    indexes.push(n);
                },
                Memory::Heap { ptr, .. } |
                Memory::List { ptr, .. } => {
                    memories.push(Memory::SSA(ptr));
                },
                Memory::Return | Memory::Global(_) => {},
            }
        }

        indexes
    }
}

pub fn lower<'hir, 'mir>(mir_session: MirSession<'hir, 'mir>) -> Session<'hir, 'mir> {
    let mut session = Session::from_mir(mir_session.clone());
    let mut lets = Vec::with_capacity(mir_session.lets.len());
    let mut funcs = Vec::with_capacity(mir_session.funcs.len());
    let mut asserts = Vec::with_capacity(mir_session.asserts.len());

    for r#let in mir_session.lets.iter() {
        lets.push(Let::from_mir(r#let, &mut session));
    }

    for func in mir_session.funcs.iter() {
        funcs.push(Func::from_mir(func, &mut session));
    }

    for assert in mir_session.asserts.iter() {
        asserts.push(Assert::from_mir(assert, &mut session, true /* is_top_level */));
    }

    session.lets = lets;
    session.funcs = funcs;
    session.asserts = asserts;

    session
}
