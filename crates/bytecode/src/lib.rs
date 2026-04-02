use sodigy_mir::{Intrinsic, Session as MirSession};
use sodigy_span::Span;

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
pub use executable::Executable;
pub(crate) use expr::lower_expr;
pub use func::Func;
pub use r#let::Let;
pub use session::{LocalValue, Session};
pub use value::Value;

/// It assumes that the runtime doesn't increment the ref_count after creating a new object.
/// For example, `Bytecode::Const` must create an object with ref_count = 0, so it'll append
/// `Bytecode::IncRefCount` after `Bytecode::Const` depending on the type of the object.
#[derive(Clone, Debug)]
pub enum Bytecode {
    Const {
        value: Value,
        dst: Memory,
    },
    Move {
        src: Memory,
        dst: Memory,
    },
    Phi {
        pair: (u32, u32),
        dst: Memory,
    },

    Jump(Label),

    Call {
        func: Label,
        args: Vec<u32>,  // list of SSA indexes
        tail: bool,
    },

    CallDynamic {
        func: Memory,    // function pointer
        args: Vec<u32>,  // list of SSA indexes
        tail: bool,
    },

    // Jumps if the `value` is 1.
    JumpIf {
        value: Memory,
        label: Label,
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

    Return(u32 /* ssa reg */),

    Intrinsic {
        intrinsic: Intrinsic,
        args: Vec<u32>,  // list of SSA indexes

        // The result of the intrinsic, if exists, will be stored here.
        dst: Memory,
    },

    // `InitTuple` and `InitList` are very similar.
    // It allocates a heap memory and saves the pointer to `dst`.
    // In runtime's point of view, tuples and structs are the same.
    // So the compiler emits `InitTuple` to initialize a struct.
    InitTuple {
        elements: usize,
        dst: Memory,
    },
    InitList {
        elements: usize,
        dst: Memory,
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
    // When a function is called and returned, the function's return value is stored here.
    Return,
    SSA(u32),

    Heap {
        ptr: Box<Memory>,
        offset: Offset,
    },

    // `Memory::Heap` and `Memory::List` may or may not be identical.
    // It just gives more hints to the runtime so that the runtime can
    // do optimizations for lists.
    List {
        ptr: Box<Memory>,
        offset: Offset,
    },

    // Top-level `let` statements.
    Global(Span),
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
