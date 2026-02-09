use sodigy_mir::{Intrinsic, Session as MirSession};
use sodigy_span::Span;

mod assert;
mod endec;
mod executable;
mod expr;
mod func;
mod r#let;
mod session;
mod value;

pub use assert::Assert;
pub use executable::Executable;
pub(crate) use expr::lower_expr;
pub use func::Func;
pub use r#let::Let;
pub use session::{LocalValue, Session};
pub use value::Value;

#[derive(Clone, Debug)]
pub enum Bytecode {
    Const {
        value: Value,
        dst: Memory,
    },
    Move {
        src: Memory,
        dst: Memory,
        inc_rc: bool,
    },

    Update {
        // a compound value
        src: Memory,

        // an integer (it updates the nth element of the compound value)
        offset: Offset,

        // it'll be a new member of the compound value
        value: Memory,

        // where to store the updated compound value
        dst: InPlaceOrMemory,
    },
    Read {
        // the compound value
        src: Memory,

        // an integer (it reads the nth element of the compound value)
        offset: Offset,

        // where to store the read value
        dst: Memory,
    },

    IncStackPointer(usize),
    DecStackPointer(usize),

    // TODO: drop semantics
    Drop(Memory),

    Jump(Label),

    // There's a function pointer in `Memory`. It'll jump to the function.
    JumpDynamic(Memory),

    JumpIf {
        value: Memory,
        label: Label,
    },

    // It'll jump to `Label::Global(def_span)` if `Memory::Global(def_span)` is not init.
    // Otherwise, it does nothing.
    JumpIfUninit {
        def_span: Span,

        // If you jump here, it'll evaluate the value and push the result to
        // `Memory::Global(def_span)`, then return.
        label: Label,
    },

    // Definition of a label.
    Label(Label),

    PushCallStack(Label),
    PopCallStack,

    // Jumps to `call_stack.peek()`.
    // It doesn't pop call_stack.
    Return,

    Intrinsic {
        intrinsic: Intrinsic,

        // stack[stack_pointer + stack_offset] is the first argument of the intrinsic
        stack_offset: usize,

        // The result of the intrinsic, if exists, will be stored here.
        dst: Memory,
    },

    // `InitTuple` and `InitList` are very similar.
    // It allocates a heap memory, copies the elements on stack,
    // and saves the pointer to `dst`.
    // The elements are at stack[(stack_pointer + stack_offset)..(stack_pointer + stack_offset + elements)].
    // In runtime's point of view, tuples and structs are the same. So
    // the compiler emits `InitTuple` to initialize a struct.
    InitTuple {
        stack_offset: usize,
        elements: usize,
        dst: Memory,
    },
    InitList {
        stack_offset: usize,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Memory {
    // A register that can hold single value.
    Return,

    // The runtime has a stack pointer.
    // Use Bytecode::IncStackPointer or Bytecode::DecStackPointer.
    Stack(usize /* offset */),

    // Top-level `let` statements.
    Global(Span),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Label {
    Local(u32),
    Global(Span /* def_span of the item */),

    // Labels are flattened by `Session::into_exeutable`.
    // After flattened, every label in the executable has a unique id.
    Flatten(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum Offset {
    Static(u32),
    Dynamic(Memory),
}

#[derive(Clone, Copy, Debug)]
pub enum InPlaceOrMemory {
    InPlace,
    Memory(Memory),
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
    //   1. has an arbitrary number of arguments
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

pub fn lower(mir_session: MirSession) -> Session {
    let mut session = Session::from_mir(mir_session.clone());
    let mut lets = Vec::with_capacity(mir_session.lets.len());
    let mut funcs = Vec::with_capacity(mir_session.funcs.len());
    let mut asserts = Vec::with_capacity(mir_session.asserts.len());

    for r#let in mir_session.lets.iter() {
        lets.push(Let::from_mir(r#let, &mut session));
    }

    for func in mir_session.funcs.iter() {
        if func.built_in {
            continue;
        }

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
