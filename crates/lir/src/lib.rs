use sodigy_mir::Session as MirSession;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod assert;
mod bytecode;
mod endec;
mod expr;
mod func;
mod session;

pub use assert::Assert;
pub (crate) use assert::AssertionMetadataKind;
pub(crate) use expr::lower_expr;
pub use bytecode::Bytecode;
pub use func::Func;
pub use session::Session;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Memory {
    // A register that can hold single value.
    Return,

    // The runtime has a stack pointer.
    // Use Bytecode::IncStackPointer or Bytecode::DecStackPointer.
    Stack(usize /* offset */),
}

#[derive(Clone, Copy, Debug)]
pub enum Label {
    Local(u32),
    Func(Span),
}

#[derive(Clone, Debug)]
pub enum Const {
    String(InternedString),
    Number(InternedNumber),

    // for panics and assertions
    Span(Span),
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

pub fn lower(mir_session: MirSession) -> Session {
    let mut session = Session::from_mir_session(&mir_session);
    let mut funcs = Vec::with_capacity(mir_session.funcs.len());

    for func in mir_session.funcs.iter() {
        funcs.push(Func::from_mir(func, &mut session));
    }

    // TODO: lets and asserts

    session
}
