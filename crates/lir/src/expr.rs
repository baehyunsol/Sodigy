use crate::{Bytecode, Label, Register, Session};
use sodigy_mir as mir;
use sodigy_token::InfixOp;

// It pushes the expr to `Register::Return`.
// If it's a tail-call, it jumps to another function after evaluating the expr.
pub fn lower_mir_expr(mir_expr: &mir::Expr, session: &mut Session, bytecode: &mut Vec<Bytecode>, is_tail_call: bool) {
    match mir_expr {
        mir::Expr::Identifier(id) => {
            bytecode.push(Bytecode::Push {
                src: *session.local_registers.get(&id.def_span).unwrap(),
                dst: Register::Return,
            });

            if is_tail_call {
                for i in 0..session.func_arg_count {
                    bytecode.push(Bytecode::Pop(Register::Call(i as u32)));
                }

                bytecode.push(Bytecode::Return);
            }
        },
        mir::Expr::If(mir::If { cond, true_value, false_value, .. }) => {
            let label1 = session.get_tmp_label();
            let label2 = session.get_tmp_label();
            lower_mir_expr(cond, session, bytecode, false /* is_tail_call */);
            bytecode.push(Bytecode::JumpIf {
                value: Register::Return,
                label: label1,
            });

            // If it `is_tail_call`, it'll exit after evaluating `false_value`,
            // so we don't have to care about it.
            // Otherwise, we have to skip evaluating `true_value`.
            lower_mir_expr(false_value, session, bytecode, is_tail_call);
            bytecode.push(Bytecode::Goto(label2));

            bytecode.push(Bytecode::Label(label1));
            lower_mir_expr(true_value, session, bytecode, is_tail_call);
            bytecode.push(Bytecode::Label(label2));
        },
        mir::Expr::Block(mir::Block { lets, value, .. }) => {
            for r#let in lets.iter() {
                todo!()
            }

            lower_mir_expr(value, session, bytecode, is_tail_call);
        },
        mir::Expr::Call { func, args } => {
            let func = match func {
                mir::Callable::Static { def_span, .. } => Label::Func(*def_span),
                // If type-check was successful, this branch is unreachable.
                // I'm doing this for debugging.
                mir::Callable::GenericInfixOp { op, .. } => match *op {
                    InfixOp::Add => Label::Intrinsic(mir::Intrinsic::IntegerAdd),
                    _ => panic!("TODO: {func:?}"),
                },
                _ => panic!("TODO: {func:?}"),
            };

            for (i, arg) in args.iter().enumerate() {
                lower_mir_expr(arg, session, bytecode, false /* is_tail_call */);
                bytecode.push(Bytecode::Push {
                    src: Register::Return,
                    dst: Register::Call(i as u32),
                });
            }

            if is_tail_call {
                for i in 0..session.func_arg_count {
                    bytecode.push(Bytecode::Pop(Register::Call(i as u32)));
                }

                bytecode.push(Bytecode::Goto(func));
            }

            else {
                let label = session.get_tmp_label();
                bytecode.push(Bytecode::PushCallStack(label));
                bytecode.push(Bytecode::Goto(func));
                bytecode.push(Bytecode::Label(label));
                bytecode.push(Bytecode::PopCallStack);
            }
        },
        _ => panic!("TODO: {mir_expr:?}"),
    }
}
