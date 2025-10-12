use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

mod expr;
mod func;
mod r#let;
mod session;

pub use expr::lower_mir_expr;
pub use func::lower_mir_func;
pub use r#let::lower_mir_let;
pub use session::Session;

#[derive(Clone, Copy, Debug)]
pub enum Bytecode {
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

    // creates a label
    Label(Label),

    // goto(call_stack.peek());
    // It doesn't pop `call_stack`.
    Return,

    JumpIf {
        value: Register,
        label: Label,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum Register {
    // These are stacks
    Local(u32),
    Call(u32),

    // These are not stacks
    Return,
}

#[derive(Clone, Copy, Debug)]
pub enum Label {
    Local(u32),
    Func(Span /* def_span */),
    Intrinsic(sodigy_mir::Intrinsic),
}

#[derive(Clone, Copy, Debug)]
pub enum Const {
    Number(InternedNumber),
    String(InternedString),
}

pub fn lower_mir(mir_session: &sodigy_mir::Session) -> Session {
    let mut session = Session::new();

    for func in mir_session.funcs.iter() {
        lower_mir_func(func, &mut session);
    }

    session
}

// fn add(x, y) = x + y;
/* [
    // Copy `Call(_)` values to `Local(_)`
    Push { src: Register::Call(0), dst: Register::Local(0) },
    Pop(Register::Call(0)),
    Push { src: Register::Call(1), dst: Register::Local(1) },
    Pop(Register::Call(1)),

    // Prepare call to `IntegerAdd`
    Push { src: Register::Local(0), dst: Register::Call(0) },
    Push { src: Register::Local(1), dst: Register::Call(1) },

    // Pops local values before exit
    Pop(Register::Local(0)),
    Pop(Register::Local(1)),

    // It's a tail-call
    Goto(Label::PrimitiveFunc(IntegerAdd)),
]; */

// fn fibo(n) = if n < 2 { 1 } else { fibo(n - 1) + fibo(n - 2) };
/* [
    // Copy `Call(_)` values to `Local(_)`
    Push { src: Register::Call(0), dst: Register::Local(0) },
    Pop(Register::Call(0)),

    // `n < 2`
    Push { src: Register::Local(0), dst: Register::Call(0) },
    PushConst { value: 2, dst: Register::Call(1) },
    PushCallStack(Label::L1),
    Goto(Label::PrimitiveFunc(IntegerLt)),
    Label(Label::L1),
    PopCallStack,

    // `if n < 2`
    JumpIf { value: Register::Return, label: Label::L2 },

    // `fibo(n - 1) + fibo(n - 2)`
    // `r1 = fibo(n - 1)`
    Push { src: Register::Local(0), dst: Register::Call(0) },
    PushConst { value: 1, dst: Register::Call(1) },
    PushCallStack(Label::L3),
    Goto(Label::Func(fibo)),
    Label(Label::L3),
    PopCallStack,
    Push { src: Register::Return, dst: Register::Call(0) },

    // `r2 = fibo(n - 2)`
    Push { src: Register::Local(0), dst: Register::Call(0) },
    PushConst { value: 2, dst: Register::Call(1) },
    PushCallStack(Label::L4),
    Goto(Label::Func(fibo)),
    Label(Label::L4),
    PopCallStack,
    Push { src: Register::Return, dst: Register::Call(1) },

    // `return r1 + r2;`
    // It's a tail-call
    Pop(Register::Local(0)),
    Goto(Label::PrimitiveFunc(IntegerAdd)),

    // `1`
    Label(Label::L2),
    PushConst { value: 1, dst: Register::Return },
    Pop(Register::Local(0)),
    Return,
]; */
