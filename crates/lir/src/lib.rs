use sodigy_mir::Session as MirSession;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod assert;
mod bytecode;
mod endec;
mod executable;
mod expr;
mod func;
mod r#let;
mod session;

pub use assert::Assert;
pub use bytecode::Bytecode;
pub use executable::Executable;
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

// There are only 2 types of values in sodigy runtime: scalar and compound.
// Scalar values are always 32 bits. They're not reference-counted.
// In Sodigy, there are only 2 primitive types that are scalar: `Byte` and `Char`.
// A compound value consists of 0 or more scalar or compoudn values. They're reference counted.
// In Sodigy, everything other than `Byte` and `Char` are compound.
// Some notes on compound values:
//     - Integers have arbitrary widths. Sodigy compiler knows that integers are reference-counted,
//       but doesn't care how it's implemented. It doesn't even care whether it's compound or not,
//       because it won't do `UpdateCompound` or `ReadCompound` with integers.
//     - A list with N elements is a compound value with N+1 elements. The first element
//       is an integer (Sodigy integer, not a scalar one), which is the length of the list.
//       The other elements are the elements of the list.
//       - TODO: why Sodigy integer? That's expensive!
//     - A string is just `[Char]`.
//     - There's nothing special about tuples and structs.
//     - TODO: enums...
#[derive(Clone, Copy, Debug)]
pub enum Const {
    Scalar(u32),

    // These are just compound values, but are here for optimization.
    // Imagine you're initializing a string literal with 10000 characters.
    // You don't want to generate `UpdateCompound` 10000 times, right?
    String {
        binary: bool,
        s: InternedString,
    },
    Number(InternedNumber),

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

pub fn lower(mir_session: MirSession) -> Session {
    let mut session = Session::from_mir_session(&mir_session);

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
