use crate::{
    Bytecode,
    Register,
    Session,
    lower_mir_expr,
};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub bytecode: Vec<Bytecode>,

    // After calling `session.make_labels_static`, every object will be mapped to a `Label::Static(id)`.
    // This is the id of the label.
    pub label_id: Option<u32>,
}

impl Func {
    // Before calling a function, the caller
    //    1. Pushes args to Register::Call(i).
    //    2. If it's not tail call, it pushes return address to the call stack.
    // The callee
    //    1. Copies values in `Register::Call(i)` to `Register::Local(i)`.
    //    2. Pops values in `Register::Call(i)`.
    //       - The callee is responsible for this because if it's a tail-call, it's not gonna return.
    //    3. Evaluates the result and pushes it to `Register::Return`.
    //    4. Pops values in `Register::Local(i)`.
    //    5. Peeks (not pop) the call stack and jump to there.
    // After calling a function, the caller pops the return address from the call stack.
    pub fn from_mir(mir_func: &mir::Func, session: &mut Session) -> Func {
        let mut bytecode = vec![];
        session.enter_func();
        session.func_arg_count = mir_func.args.len();

        for (i, arg) in mir_func.args.iter().enumerate() {
            let dst = session.register_local_name(arg.name_span);
            bytecode.push(Bytecode::Push {
                src: Register::Call(i as u32),
                dst,
            });
            bytecode.push(Bytecode::Pop(Register::Call(i as u32)));
        }

        lower_mir_expr(&mir_func.value, session, &mut bytecode, true /* is_tail_call */);
        Func {
            name: mir_func.name,
            name_span: mir_func.name_span,
            bytecode,
            label_id: None,
        }
    }
}
