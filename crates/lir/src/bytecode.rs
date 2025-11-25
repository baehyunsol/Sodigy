use crate::{
    AssertionMetadataKind,
    InPlaceOrMemory,
    Label,
    Memory,
    Offset,
    Value,
};
use sodigy_mir::Intrinsic;

#[derive(Clone, Debug)]
pub enum Bytecode {
    Const {
        value: Value,
        dst: Memory,
    },
    Copy {
        src: Memory,
        dst: Memory,
    },
    Move {
        src: Memory,
        dst: Memory,
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

    Goto(Label),
    JumpIf {
        value: Memory,
        label: Label,
    },
    JumpIfUninit {
        src: Memory,
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

    // The runtime has to implement a special control flow for assertions.
    // An assertion may panic, but there's no (and will never be a) way to
    // catch a panic and recover. Then how does the runtime throw an appropriate
    // error message when an assertion fails?
    //
    // 1. The runtime evaluates the name of the assertion -> it never panics.
    // 2. It pushes the name to AssertionMetadataStack.
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
    PushAssertionMetadata {
        kind: AssertionMetadataKind,
        src: Memory,
    },
}
