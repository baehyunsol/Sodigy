use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod assert;
mod bytecode;
mod expr;
mod func;
mod r#let;
mod session;

pub use assert::Assert;
pub use bytecode::Bytecode;
pub use expr::lower_mir_expr;
pub use func::Func;
pub use r#let::Let;
pub use session::Session;

// Before calling a function, the caller pushes args to `Call(i)` registers.
// The callee uses the values in `Call(i)` registers, pops them, and pushes
// its return value to `Return` register.
// The callee may use `Local(i)` registers if it needs extra registers.
// Popping `Call(i)` is callee's responsibility because otherwise we cannot
// implement tail-call. And that's why we need `Local(i)` in some cases.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    // These are stacks.
    // `Local(i)` and `Local(j)` are different stacks if `i != j`.
    Local(u32),
    Call(u32),

    // The compiler treats it like a stack, but the runtime doesn't have to.
    // If you're implementing a runtime, when you see `Bytecode::Pop(Register::Return)`,
    // you don't have to pop anything. You just have to decrement the reference count of
    // the value in `Register::Return`.
    Return,

    // Usually, it points to a top-level `let` statement.
    // The value of the `let` statement is stored in this register,
    // or null is here (if it's not initialized yet).
    Const(Span /* def_span */),
}

#[derive(Clone, Copy, Debug)]
pub enum Label {
    // A local label is unique inside a function.
    Local(u32),
    Func(Span /* def_span */),

    // top-level `let` statements
    Const(Span /* def_span */),

    // After calling `session.make_labels_static`, all the labels will be
    // lowered to `Label::Static(_)`.
    // The compiler will try to make related labels (e.g. ones in the same function)
    // close to each other.
    Static(u32),
}

// There are only 3 types of values in Sodigy Runtime.
// Number, String: scalar values. the runtime can implement in however way they want.
// Compound: it consists of 0 or more scalar or compound values.
#[derive(Clone, Copy, Debug)]
pub enum Const {
    Number(InternedNumber),
    String {
        binary: bool,
        s: InternedString,
    },

    // `Compound(n)` is a compound value with `n` values inside.
    // It doesn't initialize the inner values. You have to initialize
    // it with `Bytecode::UpdateCompound`. The compiler will never generate
    // a code that reads an uninitialized value.
    Compound(u32),
}

#[derive(Clone, Copy, Debug)]
pub enum Offset {
    Static(u32),
    Dynamic(Register),
}

#[derive(Clone, Copy, Debug)]
pub enum ConstOrRegister {
    Const(Const),
    Register(Register),
}

#[derive(Clone, Copy, Debug)]
pub enum InPlaceOrRegister {
    InPlace,
    Register(Register),
}

// It doesn't call `session.make_labels_static`. Backend has to do that.
pub fn lower_mir(mir_session: &sodigy_mir::Session) -> Session {
    let mut session = Session::new();

    for func in mir_session.funcs.iter() {
        let func = Func::from_mir(func, &mut session);
        session.funcs.push(func);
    }

    for r#let in mir_session.lets.iter() {
        let r#let = Let::from_mir(r#let, &mut session);
        session.lets.push(r#let);
    }

    for assert in mir_session.asserts.iter() {
        let assert = Assert::from_mir(assert, &mut session, true /* is_top_level */);
        session.asserts.push(assert);
    }

    session
}
