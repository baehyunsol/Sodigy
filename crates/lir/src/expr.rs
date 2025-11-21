use crate::{
    Assert,
    Bytecode,
    Const,
    Label,
    Memory,
    Session,
};
use sodigy_mir::{Block, Callable, Expr, If, Match};

// caller is responsible for inc/decrementing the stack pointer
// callee is responsible for dropping the local values

// It generates bytecodes that
//    1) evaluates the expr
//    2) pushes the value to `dst`
pub fn lower_expr(
    expr: &Expr,
    session: &mut Session,
    bytecodes: &mut Vec<Bytecode>,
    dst: Memory,
    is_tail_call: bool,
) {
    match expr {
        Expr::Identifier(id) => {
            let src = match session.local_values.get(&id.def_span) {
                Some(src) => Memory::Stack(*src),
                None => panic!("TODO: {id:?}"),
            };

            if src != dst {
                bytecodes.push(Bytecode::Copy {
                    src,
                    dst,
                })
            }

            if is_tail_call {
                session.drop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        Expr::Number { n, .. } => {
            bytecodes.push(Bytecode::Const {
                value: Const::Number(n.clone()),
                dst,
            });

            if is_tail_call {
                session.drop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        Expr::String { s, binary, .. } => todo!(),
        Expr::Char { ch, .. } => todo!(),
        Expr::Byte { b, .. } => todo!(),
        Expr::If(If { cond, true_value, false_value, .. }) => {
            let eval_true_value = session.get_local_label();
            let return_expr = session.get_local_label();
            lower_expr(
                cond,
                session,
                bytecodes,
                Memory::Return,
                /* is_tail_call: */ false,
            );
            bytecodes.push(Bytecode::JumpIf {
                value: Memory::Return,
                label: eval_true_value,
            });

            // We don't drop `cond` because it's a boolean!!

            // If it `is_tail_call`, it'll exit after evaluating `false_value`,
            // so we don't have to care about it.
            // Otherwise, we have to skip evaluating `true_value`.
            lower_expr(false_value, session, bytecodes, dst, is_tail_call);
            bytecodes.push(Bytecode::Goto(return_expr));

            bytecodes.push(Bytecode::Label(eval_true_value));
            lower_expr(true_value, session, bytecodes, dst, is_tail_call);
            bytecodes.push(Bytecode::Label(return_expr));
        },
        Expr::Match(Match { .. }) => todo!(),
        Expr::Block(Block { lets, asserts, value, .. }) => {
            let mut local_names = vec![];

            for r#let in lets.iter() {
                // The session will remember whether it should drop this value.
                let dst = session.register_local_name(r#let.name_span);
                local_names.push(r#let.name_span);
                lower_expr(
                    &r#let.value,
                    session,
                    bytecodes,
                    dst,
                    /* is_tail_call: */ false,
                );
            }

            for assert in asserts.iter() {
                Assert::from_mir(assert, session, /* is_top_level: */ false);
            }

            lower_expr(value, session, bytecodes, dst, is_tail_call);

            if !is_tail_call {
                session.drop_block(&local_names);
            }
        },
        Expr::Call { func, args, .. } => {
            for (i, arg) in args.iter().enumerate() {
                lower_expr(
                    arg,
                    session,
                    bytecodes,
                    // TODO: better place?
                    Memory::Stack(i + session.local_values.len()),
                    /* is_tail_call: */ false,
                );
            }

            match func {
                Callable::Static { def_span, .. } => match session.intrinsics.get(def_span) {
                    Some(intrinsic) => {
                        bytecodes.push(Bytecode::Intrinsic {
                            intrinsic: *intrinsic,
                            stack_offset: session.local_values.len(),
                            dst,
                        });

                        // TODO: how do we know whether we should drop the args?
                        for (i, arg) in args.iter().enumerate() {
                            // if has_to_drop(arg) {
                            //     drop Memory::Stack(i + session.local_values.len())
                            // }
                        }

                        if is_tail_call {
                            session.drop_all_locals(bytecodes);
                            bytecodes.push(Bytecode::Return);
                        }
                    },
                    None => {
                        let func = Label::Func(*def_span);

                        if is_tail_call {
                            session.drop_all_locals(bytecodes);

                            for i in 0..args.len() {
                                bytecodes.push(Bytecode::Move {
                                    src: Memory::Stack(i + session.local_values.len()),
                                    dst: Memory::Stack(i),
                                });
                            }

                            bytecodes.push(Bytecode::Goto(func));
                        }

                        else {
                            let return_label = session.get_local_label();
                            bytecodes.push(Bytecode::PushCallStack(return_label));
                            bytecodes.push(Bytecode::IncStackPointer(session.local_values.len()));
                            bytecodes.push(Bytecode::Goto(func));
                            bytecodes.push(Bytecode::Label(return_label));
                            bytecodes.push(Bytecode::DecStackPointer(session.local_values.len()));
                            bytecodes.push(Bytecode::PopCallStack);

                            if dst != Memory::Return {
                                bytecodes.push(Bytecode::Move {
                                    src: Memory::Return,
                                    dst,
                                });
                            }
                        }
                    },
                },
                _ => todo!(),
            }
        },
        _ => panic!("TODO: {expr:?}"),
    }
}
