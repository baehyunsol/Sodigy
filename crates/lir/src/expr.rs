use crate::{
    Assert,
    Bytecode,
    Const,
    Label,
    Register,
    Session,
};
use sodigy_mir::{self as mir, Callable, Intrinsic};
use sodigy_name_analysis::{
    NameKind,
    NameOrigin,
};
use sodigy_token::InfixOp;

// It pushes the expr to `Register::Return`.
// If it's a tail-call, it jumps to another function after evaluating the expr.
pub fn lower_mir_expr(mir_expr: &mir::Expr, session: &mut Session, bytecode: &mut Vec<Bytecode>, is_tail_call: bool) {
    match mir_expr {
        mir::Expr::Identifier(id) => {
            let src = match session.local_registers.get(&id.def_span) {
                Some(src) => *src,
                None => match id.origin {
                    // If a top-level statement references another top-level `let` statement,
                    // this branch is reached.
                    NameOrigin::Local { kind: NameKind::Let { is_top_level: true } } => {
                        let return_from_const = session.get_tmp_label();
                        let const_is_already_init = session.get_tmp_label();
                        // NOTE: top-level `let` statements are always lazy-evaluated.
                        bytecode.push(Bytecode::JumpIfInit {
                            reg: Register::Const(id.def_span),
                            label: const_is_already_init,
                        });

                        bytecode.push(Bytecode::PushCallStack(return_from_const));
                        bytecode.push(Bytecode::Goto(Label::Const(id.def_span)));
                        bytecode.push(Bytecode::Label(return_from_const));
                        bytecode.push(Bytecode::PopCallStack);
                        bytecode.push(Bytecode::Label(const_is_already_init));

                        Register::Const(id.def_span)
                    },
                    NameOrigin::Foreign { .. } => panic!("TODO: {id:?}"),

                    // Otherwise, it's a local value, so it must be at `session.local_registers`.
                    _ => unreachable!(),
                },
            };

            bytecode.push(Bytecode::Push {
                src,
                dst: Register::Return,
            });

            if is_tail_call {
                session.pop_all_locals(bytecode);
                bytecode.push(Bytecode::Return);
            }
        },
        mir::Expr::Number { n, .. } => {
            bytecode.push(Bytecode::PushConst {
                value: Const::Number(*n),
                dst: Register::Return,
            });

            if is_tail_call {
                session.pop_all_locals(bytecode);
                bytecode.push(Bytecode::Return);
            }
        },
        mir::Expr::String { s, binary, .. } => {
            bytecode.push(Bytecode::PushConst {
                value: Const::String { s: *s, binary: *binary },
                dst: Register::Return,
            });

            if is_tail_call {
                session.pop_all_locals(bytecode);
                bytecode.push(Bytecode::Return);
            }
        },
        mir::Expr::If(mir::If { cond, true_value, false_value, .. }) => {
            let eval_true_value = session.get_tmp_label();
            let return_expr = session.get_tmp_label();
            lower_mir_expr(cond, session, bytecode, false /* is_tail_call */);
            bytecode.push(Bytecode::JumpIf {
                value: Register::Return,
                label: eval_true_value,
            });
            bytecode.push(Bytecode::Pop(Register::Return));

            // If it `is_tail_call`, it'll exit after evaluating `false_value`,
            // so we don't have to care about it.
            // Otherwise, we have to skip evaluating `true_value`.
            lower_mir_expr(false_value, session, bytecode, is_tail_call);
            bytecode.push(Bytecode::Goto(return_expr));

            bytecode.push(Bytecode::Label(eval_true_value));
            bytecode.push(Bytecode::Pop(Register::Return));
            lower_mir_expr(true_value, session, bytecode, is_tail_call);
            bytecode.push(Bytecode::Label(return_expr));
        },
        mir::Expr::Block(mir::Block { lets, asserts, value, .. }) => {
            // TODO: it assumes that there's no dependency between `let` statements and
            //       everything is eager-evaluated.
            for r#let in lets.iter() {
                let dst = session.register_local_name(r#let.name_span);
                lower_mir_expr(&r#let.value, session, bytecode, false /* is_tail_call */);
                bytecode.push(Bytecode::Push {
                    src: Register::Return,
                    dst,
                });
                bytecode.push(Bytecode::Pop(Register::Return));
            }

            // TODO: when we have clear rules for lazy-evaluating let statements (and dependencies),
            //       we might have to modify this
            for assert in asserts.iter() {
                let assert = Assert::from_mir(assert, session, false /* is_top_level */);
                bytecode.extend(assert.bytecode);
            }

            lower_mir_expr(value, session, bytecode, is_tail_call);
        },
        mir::Expr::Call { func, args } => {
            for (i, arg) in args.iter().enumerate() {
                lower_mir_expr(arg, session, bytecode, false /* is_tail_call */);
                bytecode.push(Bytecode::Push {
                    src: Register::Return,
                    dst: Register::Call(i as u32),
                });
                bytecode.push(Bytecode::Pop(Register::Return));
            }

            match func {
                Callable::Static { def_span, .. } => {
                    let func = Label::Func(*def_span);

                    if is_tail_call {
                        session.pop_all_locals(bytecode);
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
                // If type-check was successful, `Callable::GenericInfixOp` is unreachable.
                // I'm doing this for debugging.
                Callable::GenericInfixOp { .. } |
                Callable::Intrinsic { .. } => {
                    let intrinsic = match func {
                        Callable::GenericInfixOp { op, .. } => match op {
                            InfixOp::Add => Intrinsic::IntegerAdd,
                            InfixOp::Sub => Intrinsic::IntegerSub,
                            InfixOp::Eq => Intrinsic::IntegerEq,
                            InfixOp::Lt => Intrinsic::IntegerLt,
                            _ => panic!("TODO: {op:?}"),
                        },
                        Callable::Intrinsic { intrinsic, .. } => *intrinsic,
                        _ => unreachable!(),
                    };
                    bytecode.push(Bytecode::Intrinsic(intrinsic));

                    for i in 0..args.len() {
                        bytecode.push(Bytecode::Pop(Register::Call(i as u32)));
                    }

                    if is_tail_call {
                        session.pop_all_locals(bytecode);
                        bytecode.push(Bytecode::Return);
                    }
                },
                _ => panic!("TODO: {func:?}"),
            }
        },
        _ => panic!("TODO: {mir_expr:?}"),
    }
}
