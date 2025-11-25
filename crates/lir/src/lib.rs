use sodigy_mir::Session as MirSession;
use sodigy_span::Span;

mod assert;
mod bytecode;
mod endec;
mod expr;
mod func;
mod r#let;
mod session;
mod value;

pub use assert::Assert;
pub (crate) use assert::AssertionMetadataKind;
pub(crate) use expr::lower_expr;
pub use bytecode::Bytecode;
pub use func::Func;
pub use r#let::Let;
pub use session::Session;
pub use value::Value;

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
    let mut session = Session::from_mir(&mir_session);
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
