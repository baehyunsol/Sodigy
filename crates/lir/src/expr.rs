use crate::{
    Assert,
    Bytecode,
    Const,
    ConstOrRegister,
    InPlaceOrRegister,
    Label,
    Offset,
    Register,
    Session,
};
use sodigy_mir::{self as mir, Callable, Intrinsic};
use sodigy_name_analysis::{
    NameKind,
    NameOrigin,
};
use sodigy_number::InternedNumber;
use sodigy_token::InfixOp;

// It pushes the expr to `Register::Return`.
// If it's a tail-call, it jumps to another function after evaluating the expr.
pub fn lower_mir_expr(mir_expr: &mir::Expr, session: &mut Session, bytecodes: &mut Vec<Bytecode>, is_tail_call: bool) {
    match mir_expr {
        mir::Expr::Identifier(id) => {
            let src = match session.local_registers.get(&id.def_span) {
                Some(src) => *src,
                None => match id.origin {
                    // top-level `let` statements are always lazily evaluated.
                    NameOrigin::Local { kind: NameKind::Let { is_top_level: true } } |
                    NameOrigin::Foreign { kind: NameKind::Let { is_top_level: true } } => {
                        let return_from_const = session.get_tmp_label();
                        let const_is_already_init = session.get_tmp_label();
                        // NOTE: top-level `let` statements are always lazy-evaluated.
                        bytecodes.push(Bytecode::JumpIfInit {
                            reg: Register::Const(id.def_span),
                            label: const_is_already_init,
                        });

                        bytecodes.push(Bytecode::PushCallStack(return_from_const));
                        bytecodes.push(Bytecode::Goto(Label::Const(id.def_span)));
                        bytecodes.push(Bytecode::Label(return_from_const));
                        bytecodes.push(Bytecode::PopCallStack);
                        bytecodes.push(Bytecode::Label(const_is_already_init));

                        Register::Const(id.def_span)
                    },
                    NameOrigin::Foreign { .. } => panic!("TODO: {id:?}"),

                    // Otherwise, it's a local value, so it must be at `session.local_registers`.
                    _ => unreachable!(),
                },
            };

            bytecodes.push(Bytecode::Push {
                src,
                dst: Register::Return,
            });

            if is_tail_call {
                session.pop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        mir::Expr::Number { n, .. } => {
            bytecodes.push(Bytecode::PushConst {
                value: Const::Number(*n),
                dst: Register::Return,
            });

            if is_tail_call {
                session.pop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        mir::Expr::String { s, binary, .. } => {
            bytecodes.push(Bytecode::PushConst {
                value: Const::String { s: *s, binary: *binary },
                dst: Register::Return,
            });

            if is_tail_call {
                session.pop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        mir::Expr::If(mir::If { cond, true_value, false_value, .. }) => {
            let eval_true_value = session.get_tmp_label();
            let return_expr = session.get_tmp_label();
            lower_mir_expr(cond, session, bytecodes, false /* is_tail_call */);
            bytecodes.push(Bytecode::JumpIf {
                value: Register::Return,
                label: eval_true_value,
            });
            bytecodes.push(Bytecode::Pop(Register::Return));

            // If it `is_tail_call`, it'll exit after evaluating `false_value`,
            // so we don't have to care about it.
            // Otherwise, we have to skip evaluating `true_value`.
            lower_mir_expr(false_value, session, bytecodes, is_tail_call);
            bytecodes.push(Bytecode::Goto(return_expr));

            bytecodes.push(Bytecode::Label(eval_true_value));
            bytecodes.push(Bytecode::Pop(Register::Return));
            lower_mir_expr(true_value, session, bytecodes, is_tail_call);
            bytecodes.push(Bytecode::Label(return_expr));
        },
        mir::Expr::Block(mir::Block { lets, asserts, value, .. }) => {
            // TODO: it assumes that there's no dependency between `let` statements and
            //       everything is eager-evaluated.
            for r#let in lets.iter() {
                let dst = session.register_local_name(r#let.name_span);
                lower_mir_expr(&r#let.value, session, bytecodes, false /* is_tail_call */);
                bytecodes.push(Bytecode::Push {
                    src: Register::Return,
                    dst,
                });
                bytecodes.push(Bytecode::Pop(Register::Return));
            }

            // TODO: when we have clear rules for lazy-evaluating let statements (and dependencies),
            //       we might have to modify this
            for assert in asserts.iter() {
                let assert = Assert::from_mir(assert, session, false /* is_top_level */);
                bytecodes.extend(assert.bytecodes);
            }

            lower_mir_expr(value, session, bytecodes, is_tail_call);
        },
        mir::Expr::Call { func, args, .. } => {
            for (i, arg) in args.iter().enumerate() {
                lower_mir_expr(arg, session, bytecodes, false /* is_tail_call */);
                bytecodes.push(Bytecode::Push {
                    src: Register::Return,
                    dst: Register::Call(i as u32),
                });
                bytecodes.push(Bytecode::Pop(Register::Return));
            }

            match func {
                Callable::Static { def_span, .. } => {
                    let func = Label::Func(*def_span);

                    if is_tail_call {
                        session.pop_all_locals(bytecodes);
                        bytecodes.push(Bytecode::Goto(func));
                    }

                    else {
                        let label = session.get_tmp_label();
                        bytecodes.push(Bytecode::PushCallStack(label));
                        bytecodes.push(Bytecode::Goto(func));
                        bytecodes.push(Bytecode::Label(label));
                        bytecodes.push(Bytecode::PopCallStack);
                    }
                },
                Callable::TupleInit { .. } => {
                    bytecodes.push(Bytecode::PushConst {
                        value: Const::Compound(args.len() as u32),
                        dst: Register::Return,
                    });

                    for i in 0..args.len() {
                        bytecodes.push(Bytecode::UpdateCompound {
                            src: Register::Return,
                            offset: Offset::Static(i as u32),
                            value: ConstOrRegister::Register(Register::Call(i as u32)),
                            dst: InPlaceOrRegister::InPlace,
                        });
                        bytecodes.push(Bytecode::Pop(Register::Call(i as u32)));
                    }

                    if is_tail_call {
                        session.pop_all_locals(bytecodes);
                        bytecodes.push(Bytecode::Return);
                    }
                },
                // The first element is the length of the list.
                Callable::ListInit { .. } => {
                    bytecodes.push(Bytecode::PushConst {
                        value: Const::Compound(args.len() as u32 + 1),
                        dst: Register::Return,
                    });

                    bytecodes.push(Bytecode::UpdateCompound {
                        src: Register::Return,
                        offset: Offset::Static(0),
                        value: ConstOrRegister::Const(Const::Number(InternedNumber::from_u32(args.len() as u32, true /* is_integer */))),
                        dst: InPlaceOrRegister::InPlace,
                    });

                    for i in 0..args.len() {
                        bytecodes.push(Bytecode::UpdateCompound {
                            src: Register::Return,
                            offset: Offset::Static(i as u32 + 1),
                            value: ConstOrRegister::Register(Register::Call(i as u32)),
                            dst: InPlaceOrRegister::InPlace,
                        });
                        bytecodes.push(Bytecode::Pop(Register::Call(i as u32)));
                    }

                    if is_tail_call {
                        session.pop_all_locals(bytecodes);
                        bytecodes.push(Bytecode::Return);
                    }
                },
                // If type-check was successful, `Callable::GenericInfixOp` is unreachable.
                // But the type-checker isn't complete yet. I'm doing this for debugging.
                Callable::GenericInfixOp { op, .. } => match op {
                    InfixOp::Add |
                    InfixOp::Sub |
                    InfixOp::Mul |
                    InfixOp::Div |
                    InfixOp::Eq |
                    InfixOp::Gt |
                    InfixOp::Lt => {
                        let intrinsic = match op {
                            InfixOp::Add => Intrinsic::IntegerAdd,
                            InfixOp::Sub => Intrinsic::IntegerSub,
                            InfixOp::Mul => Intrinsic::IntegerMul,
                            InfixOp::Div => Intrinsic::IntegerDiv,
                            InfixOp::Eq => Intrinsic::IntegerEq,
                            InfixOp::Gt => Intrinsic::IntegerGt,
                            InfixOp::Lt => Intrinsic::IntegerLt,
                            _ => panic!("TODO: {op:?}"),
                        };
                        bytecodes.push(Bytecode::Intrinsic(intrinsic));

                        for i in 0..args.len() {
                            bytecodes.push(Bytecode::Pop(Register::Call(i as u32)));
                        }

                        if is_tail_call {
                            session.pop_all_locals(bytecodes);
                            bytecodes.push(Bytecode::Return);
                        }
                    },
                    // Call(0): list, Call(1): index
                    // It has to read `index + 1` because `ls[0]` is for the length of the list
                    InfixOp::Index => {
                        bytecodes.push(Bytecode::PushConst {
                            value: Const::Number(InternedNumber::from_u32(1, true)),
                            dst: Register::Call(0),
                        });
                        bytecodes.push(Bytecode::Intrinsic(Intrinsic::IntegerAdd));
                        bytecodes.push(Bytecode::Pop(Register::Call(0)));
                        bytecodes.push(Bytecode::Pop(Register::Call(1)));
                        bytecodes.push(Bytecode::Push {
                            src: Register::Return,
                            dst: Register::Call(1),
                        });

                        bytecodes.push(Bytecode::ReadCompound {
                            src: Register::Call(0),
                            offset: Offset::Dynamic(Register::Call(1)),
                            dst: Register::Return,
                        });

                        for i in 0..args.len() {
                            bytecodes.push(Bytecode::Pop(Register::Call(i as u32)));
                        }

                        if is_tail_call {
                            session.pop_all_locals(bytecodes);
                            bytecodes.push(Bytecode::Return);
                        }
                    },
                    _ => panic!("TODO: {op:?}"),
                },
                Callable::Intrinsic { intrinsic, .. } => {
                    bytecodes.push(Bytecode::Intrinsic(*intrinsic));

                    for i in 0..args.len() {
                        bytecodes.push(Bytecode::Pop(Register::Call(i as u32)));
                    }

                    if is_tail_call {
                        session.pop_all_locals(bytecodes);
                        bytecodes.push(Bytecode::Return);
                    }
                },
                _ => panic!("TODO: {func:?}"),
            }
        },
        _ => panic!("TODO: {mir_expr:?}"),
    }
}
