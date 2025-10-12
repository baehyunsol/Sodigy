use crate::{
    Bytecode,
    Register,
    Session,
    lower_mir_expr,
};
use sodigy_mir as mir;

// Before calling a function, the caller
//    1. Pushes args to Register::Call(i).
//    2. If it's not tail call, it pushes return address to the call stack.
// The callee
//    1. Copies values in `Register::Call(i)` to `Register::Local(i)`.
//    2. Pops values in `Register::Call(i)`.
//    3. Evaluates the result and pushes it to `Register::Return`.
//    4. Pops values in `Register::Local(i)`.
//    5. Peeks (not pop) the call stack and jump to there.
// After calling a function, the caller pops the return address from the call stack.

// It doesn't return `Vec<Bytecode>` and instead directly inserts to `session.funcs` because
// `mir::Func` is always at top-level.
pub fn lower_mir_func(mir_func: &mir::Func, session: &mut Session) {
    let mut bytecode = vec![];
    session.enter_func();
    session.func_arg_count = mir_func.args.len();

    for (i, arg) in mir_func.args.iter().enumerate() {
        let dst = session.register_local_name(arg.name_span);
        bytecode.push(Bytecode::Push {
            src: Register::Call(i as u32),
            dst: dst,
        });
        bytecode.push(Bytecode::Pop(Register::Call(i as u32)));
    }

    lower_mir_expr(&mir_func.value, session, &mut bytecode, true /* is_tail_call */);
    session.funcs.insert(mir_func.name, bytecode);
}
