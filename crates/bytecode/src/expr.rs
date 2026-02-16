use crate::{
    Assert,
    Bytecode,
    Label,
    Memory,
    Offset,
    Session,
    Value,
};
use sodigy_mir::{Block, Callable, Expr, If, Match};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_parse::Field;

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
        Expr::Ident(id) => {
            let src = match session.local_values.get(&id.def_span) {
                Some(src) => Memory::Stack(src.stack_offset),
                None => match id.origin {
                    NameOrigin::Foreign { kind } | NameOrigin::Local { kind } => match kind {
                        NameKind::EnumVariant { .. } => {
                            if session.get_lang_item_span("variant.Bool.True") == id.def_span {
                                // TODO: How do I force that every `Bool.True` is represented like this?
                                bytecodes.push(Bytecode::Const {
                                    value: Value::Scalar(1),
                                    dst,
                                });

                                if is_tail_call {
                                    session.drop_all_locals(bytecodes);
                                    bytecodes.push(Bytecode::Return);
                                }

                                return;
                            }

                            else if session.get_lang_item_span("variant.Bool.False") == id.def_span {
                                // TODO: How do I force that every `Bool.False` is represented like this?
                                bytecodes.push(Bytecode::Const {
                                    value: Value::Scalar(0),
                                    dst,
                                });

                                if is_tail_call {
                                    session.drop_all_locals(bytecodes);
                                    bytecodes.push(Bytecode::Return);
                                }

                                return;
                            }

                            else {
                                todo!()
                            }
                        },
                        NameKind::Let { is_top_level: true } => {
                            let value_inited = session.get_local_label();
                            bytecodes.push(Bytecode::PushCallStack(value_inited));
                            bytecodes.push(Bytecode::JumpIfUninit {
                                def_span: id.def_span,
                                label: Label::Global(id.def_span),
                            });
                            bytecodes.push(Bytecode::Label(value_inited));
                            bytecodes.push(Bytecode::PopCallStack);
                            // TODO: inc_ref_count if it has to
                            Memory::Global(id.def_span)
                        },
                        NameKind::Func => {
                            bytecodes.push(Bytecode::Const {
                                value: Value::FuncPointer {
                                    def_span: id.def_span,

                                    // `Session::into_executable()` will fill this
                                    program_counter: None,
                                },
                                dst,
                            });

                            if is_tail_call {
                                session.drop_all_locals(bytecodes);
                                bytecodes.push(Bytecode::Return);
                            }

                            return;
                        },
                        _ => panic!("TODO: {id:?}"),
                    },
                    _ => unreachable!(),
                },
            };

            if src != dst {
                bytecodes.push(Bytecode::Move {
                    src,
                    dst,
                })

                // TODO: inc_ref_count if it has to
            }

            if is_tail_call {
                session.drop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        Expr::Constant(c) => {
            let value = session.lower_constant(c);
            bytecodes.push(Bytecode::Const { value, dst });

            // TODO: inc_ref_count if it has to

            if is_tail_call {
                session.drop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
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

            if !is_tail_call {
                bytecodes.push(Bytecode::Jump(return_expr));
            }

            bytecodes.push(Bytecode::Label(eval_true_value));
            lower_expr(true_value, session, bytecodes, dst, is_tail_call);

            if !is_tail_call {
                bytecodes.push(Bytecode::Label(return_expr));
            }
        },
        Expr::Match(Match { .. }) => unreachable!(),
        Expr::Block(Block { lets, asserts, value, .. }) => {
            let mut local_names = vec![];

            for r#let in lets.iter() {
                let dst = Memory::Stack(session.local_values.get(&r#let.name_span).unwrap().stack_offset);
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
                bytecodes.extend(Assert::from_mir(assert, session, /* is_top_level: */ false).bytecodes);
            }

            lower_expr(value, session, bytecodes, dst, is_tail_call);

            if !is_tail_call {
                session.drop_block(&local_names);
            }
        },
        Expr::Field { lhs, fields } => {
            lower_expr(
                lhs,
                session,
                bytecodes,
                Memory::Return,
                /* is_tail_call: */ false,
            );

            for field in fields.iter() {
                match field {
                    Field::Index(i) => {
                        // TODO: drop `Memory::Return` if it has to

                        bytecodes.push(Bytecode::Read {
                            src: Memory::Return,
                            // NOTE: There are no negative index because post-mir already lowered them
                            offset: Offset::Static(*i as u32),
                            dst: Memory::Return,
                        });

                        // TODO: inc_ref_count if it has to
                    },
                    Field::Constructor => {
                        // nop
                    },
                    _ => todo!(),
                }
            }

            if dst != Memory::Return {
                bytecodes.push(Bytecode::Move {
                    src: Memory::Return,
                    dst,
                });
            }

            if is_tail_call {
                session.drop_all_locals(bytecodes);
                bytecodes.push(Bytecode::Return);
            }
        },
        Expr::Call { func, args, .. } => {
            let stack_offset = session.stack_offset;
            session.stack_offset += args.len();

            for (i, arg) in args.iter().enumerate() {
                lower_expr(
                    arg,
                    session,
                    bytecodes,
                    Memory::Stack(i + stack_offset),
                    /* is_tail_call: */ false,
                );
            }

            session.stack_offset -= args.len();

            match func {
                Callable::Static { def_span, .. } => match session.intrinsics.get(def_span) {
                    Some(intrinsic) => {
                        bytecodes.push(Bytecode::Intrinsic {
                            intrinsic: *intrinsic,
                            stack_offset,
                            dst,
                        });

                        // TODO: inc_ref_count(dst) if it has to

                        // TODO: how do we know whether we should drop the args?
                        for (i, arg) in args.iter().enumerate() {
                            // if has_to_drop(arg) {
                            //     drop Memory::Stack(i + stack_offset)
                            // }
                        }

                        if is_tail_call {
                            session.drop_all_locals(bytecodes);
                            bytecodes.push(Bytecode::Return);
                        }
                    },
                    None => {
                        let func = Label::Global(*def_span);

                        if is_tail_call {
                            session.drop_all_locals(bytecodes);

                            for i in 0..args.len() {
                                bytecodes.push(Bytecode::Move {
                                    src: Memory::Stack(i + stack_offset),
                                    dst: Memory::Stack(i),
                                });
                            }

                            bytecodes.push(Bytecode::Jump(func));
                        }

                        else {
                            let return_label = session.get_local_label();
                            bytecodes.push(Bytecode::PushCallStack(return_label));
                            bytecodes.push(Bytecode::IncStackPointer(stack_offset));
                            bytecodes.push(Bytecode::Jump(func));
                            bytecodes.push(Bytecode::Label(return_label));
                            bytecodes.push(Bytecode::DecStackPointer(stack_offset));
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
                Callable::StructInit { .. } | Callable::TupleInit { .. } | Callable::ListInit { .. } => {
                    let bytecode = match func {
                        Callable::StructInit { .. } | Callable::TupleInit { .. } => Bytecode::InitTuple {
                            stack_offset,
                            elements: args.len(),
                            dst,
                        },
                        Callable::ListInit { .. } => Bytecode::InitList {
                            stack_offset,
                            elements: args.len(),
                            dst,
                        },
                        _ => unreachable!(),
                    };
                    bytecodes.push(bytecode);
                    bytecodes.push(Bytecode::IncRefCount(dst));

                    // TODO: how do we know whether we should drop the args?
                    for (i, arg) in args.iter().enumerate() {
                        // if has_to_drop(arg) {
                        //     drop Memory::Stack(i + stack_offset)
                        // }
                    }

                    if is_tail_call {
                        session.drop_all_locals(bytecodes);
                        bytecodes.push(Bytecode::Return);
                    }
                },
                Callable::Dynamic(f) => {
                    lower_expr(
                        f,
                        session,
                        bytecodes,
                        Memory::Return,
                        /* is_tail_call: */ false,
                    );

                    if is_tail_call {
                        session.drop_all_locals(bytecodes);

                        for i in 0..args.len() {
                            bytecodes.push(Bytecode::Move {
                                src: Memory::Stack(i + stack_offset),
                                dst: Memory::Stack(i),
                            });
                        }

                        // `.drop_all_locals` doesn't drop the function pointer because a
                        // function pointer is scalar.
                        bytecodes.push(Bytecode::JumpDynamic(Memory::Return));
                    }

                    else {
                        let return_label = session.get_local_label();
                        bytecodes.push(Bytecode::PushCallStack(return_label));
                        bytecodes.push(Bytecode::IncStackPointer(stack_offset));
                        bytecodes.push(Bytecode::JumpDynamic(Memory::Return));
                        bytecodes.push(Bytecode::Label(return_label));
                        bytecodes.push(Bytecode::DecStackPointer(stack_offset));
                        bytecodes.push(Bytecode::PopCallStack);

                        if dst != Memory::Return {
                            bytecodes.push(Bytecode::Move {
                                src: Memory::Return,
                                dst,
                            });
                        }
                    }
                },
            }
        },
        _ => panic!("TODO: {expr:?}"),
    }
}
