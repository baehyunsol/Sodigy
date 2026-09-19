use sodigy_error::FuncEffect;
use sodigy_mir::{Intrinsic, Session as MirSession};
use sodigy_span::{Span, SpanHash};
use sodigy_utils::camel_to_snake;
use std::collections::HashMap;

mod assert;
mod dump;
mod endec;
mod expr;
mod expr_hash;
mod func;
mod label;
mod r#let;
mod link;
mod object_file;
mod parse;
mod session;
mod value;

#[cfg(test)]
mod tests;

pub use assert::Assert;
pub use expr_hash::ExprHash;
pub(crate) use expr::lower_expr;
pub use func::Func;
pub use label::{GlobalLabel, LocalLabel};
pub use r#let::Let;
pub use link::link;
pub use object_file::{
    BasicBlock,
    CodeKind,
    CodeSection,
    ObjectFile,
    Terminator,
};
pub use parse::{BytecodeParseError, parse as parse_bytecode};
pub use session::{LocalValue, Session};
pub use value::{InternedValue, Value};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SSA(u32);

impl SSA {
    pub fn from_u32(n: u32) -> SSA {
        SSA(n)
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

// `debug_info` fields are set only if the session's `debug_info` field is set.

#[derive(Clone, Debug)]
pub enum Bytecode {
    Const {
        value: InternedValue,
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

    Jump(LocalLabel),

    Call {
        func: GlobalLabel,
        args: Vec<SSA>,

        // The returned value is stored here.
        // It's None iff it's a tail-call.
        dst: Option<Memory>,

        debug_info: Option<Box<Span>>,

        // This information is used by the optimizer.
        effect: Box<FuncEffect>,
    },

    CallDynamic {
        func: SSA,    // function pointer
        args: Vec<SSA>,

        // The returned value is stored here.
        // It's None iff it's a tail-call.
        dst: Option<Memory>,

        debug_info: Option<Box<Span>>,

        // This information is used by the optimizer.
        effect: Box<FuncEffect>,
    },

    // Jumps if the `value` is non-zero.
    JumpIf {
        value: SSA,
        t: LocalLabel,
        f: LocalLabel,
        debug_info: Option<Box<Span>>,
    },

    // If the global value is not initialized, it calls the function `global`.
    // The function will initialize the global value and return. Then, it jumps to `label`.
    // If it's already initialized, it just jumps to `label`.
    TryInitGlobal {
        global: GlobalLabel,
        label: LocalLabel,
    },

    // If the global value is not initialized yet, it's UB.
    // There used to be `Memory::Global` and I used `Bytecode::Move` to
    // initialize global values, but when implementing the interpreter,
    // I realized that global values are very different from the other values
    // so I just created new bytecodes.
    LoadGlobal {
        src: GlobalLabel,
        dst: SSA,
    },
    StoreGlobal {
        src: SSA,
        dst: GlobalLabel,
    },

    // Definition of a label.
    Label(LocalLabel),

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
        offset: u32,
    },

    // `Memory::Heap` and `Memory::List` may or may not be identical.
    // It just gives more hints to the runtime so that the runtime can
    // do optimizations for lists.
    List {
        ptr: SSA,
        offset: u32,
    },

    // values written here will be discarded immediately
    // reading this value is UB
    Null,
}

impl Memory {
    pub fn get_heap_index(&self) -> Option<(SSA, u32)> {
        match self {
            Memory::Heap { ptr, offset } |
            Memory::List { ptr, offset } => Some((*ptr, *offset)),
            _ => None,
        }
    }
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
    pub fn get_dst(&self) -> Option<Memory> {
        match self {
            Bytecode::Const { dst, .. } |
            Bytecode::Move { dst, .. } |
            Bytecode::Phi { dst, .. } |
            Bytecode::Update { dst, .. } |
            Bytecode::Intrinsic { dst, .. } |
            Bytecode::InitTuple { dst, .. } |
            Bytecode::InitList { dst, .. } => Some(dst.clone()),
            Bytecode::Call { dst, .. } |
            Bytecode::CallDynamic { dst, .. } => dst.clone(),
            Bytecode::LoadGlobal { dst, .. } => Some(Memory::SSA(*dst)),
            Bytecode::Jump(_) |
            Bytecode::JumpIf { .. } |
            Bytecode::TryInitGlobal { .. } |
            Bytecode::StoreGlobal { .. } |
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
                Memory::Heap { ptr: a, offset: b } => {
                    if let Some(c) = heap_ssa_alias.get(&(*a, *b)) {
                        *src = Memory::SSA(*ssa_alias.get(c).unwrap_or(c));
                    }

                    else {
                        *a = *ssa_alias.get(a).unwrap_or(a);
                    }
                },
                Memory::List { ptr: a, offset: b } => {
                    if let Some(c) = heap_ssa_alias.get(&(*a, *b)) {
                        *src = Memory::SSA(*ssa_alias.get(c).unwrap_or(c));
                    }

                    else {
                        *a = *ssa_alias.get(a).unwrap_or(a);
                    }
                },
                Memory::Null => {},
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
                *func = *ssa_alias.get(func).unwrap_or(&func);
                apply_ssa_alias_args(args, ssa_alias, heap_ssa_alias);
            },
            Bytecode::TryInitGlobal { .. } => {},
            Bytecode::LoadGlobal { dst, .. } => {
                *dst = *ssa_alias.get(dst).unwrap_or(dst);
            },
            Bytecode::StoreGlobal { src, .. } => {
                *src = *ssa_alias.get(src).unwrap_or(src);
            },
            Bytecode::Label(_) => {},
            Bytecode::JumpIf { value: a, .. } |
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
            Bytecode::JumpIf { value, .. } |
            Bytecode::Return(value) => {
                indexes.push(*value);
            },
            Bytecode::LoadGlobal { dst: ssa, .. } |
            Bytecode::StoreGlobal { src: ssa, .. } => {
                indexes.push(*ssa);
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
            Bytecode::TryInitGlobal { .. } |
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
                Memory::Return | Memory::Null => {},
            }
        }

        indexes
    }

    // Whether it's okay for the optimizer to remove this bytecode.
    pub fn is_observable(&self) -> bool {
        match self {
            Bytecode::Const { .. } |
            Bytecode::Move { .. } |
            Bytecode::Phi { .. } |
            Bytecode::Jump(_) |
            Bytecode::JumpIf { .. } |

            // The top-level lets are always pure: there're no side effects.
            Bytecode::TryInitGlobal { .. } |

            Bytecode::LoadGlobal { .. } |
            Bytecode::StoreGlobal { .. } |
            Bytecode::Label(_) |
            Bytecode::Return(_) |
            Bytecode::Update { .. } |
            Bytecode::InitTuple { .. } |
            Bytecode::InitList { .. } |
            Bytecode::PushDebugInfo { .. } |
            Bytecode::PopDebugInfo => false,
            Bytecode::Call { effect, .. } |
            Bytecode::CallDynamic { effect, .. } => matches!(&**effect, FuncEffect::Proc | FuncEffect::NdetProc),
            Bytecode::Intrinsic { intrinsic, .. } => matches!(intrinsic.effect(), FuncEffect::Proc | FuncEffect::NdetProc),
        }
    }
}

pub fn lower<'hir, 'mir>(
    mir_session: MirSession<'hir, 'mir>,
    lower_built_ins: bool,
) -> Session<'hir, 'mir> {
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

    session.object_file = ObjectFile::new(
        &mut lets,
        &mut funcs,
        &mut asserts,
        &mut session.data_section,
        &mir_session.intermediate_dir,
    );

    // We need code sections for built-ins when we want to create function pointers
    // for built-in functions.
    if lower_built_ins {
        for (intrinsic, lang_item) in Intrinsic::ALL_WITH_LANG_ITEM.iter() {
            let def_span = mir_session.global_context.get_lang_item_span(lang_item);
            let label = GlobalLabel::new(def_span.hash());
            session.object_file.code.insert(
                label,
                CodeSection {
                    label,
                    span: Some(def_span.clone()),
                    kind: CodeKind::Func,
                    name: camel_to_snake(&format!("{intrinsic:?}")),
                    params: Some(intrinsic.num_params()),
                    effect: intrinsic.effect(),
                    basic_blocks: [(
                        LocalLabel::start(),
                        BasicBlock {
                            label: LocalLabel::start(),
                            code: vec![
                                Bytecode::Intrinsic {
                                    intrinsic: *intrinsic,
                                    args: (0..intrinsic.num_params()).map(
                                        |i| SSA(i as u32)
                                    ).collect(),
                                    dst: Memory::SSA(SSA(intrinsic.num_params() as u32 + 1)),
                                    debug_info: None,
                                },
                            ],
                            terminator: Terminator::Return(SSA(intrinsic.num_params() as u32 + 1)),
                            terminator_debug_info: None,
                        },
                    )].into_iter().collect(),
                },
            );
        }
    }

    session
}
