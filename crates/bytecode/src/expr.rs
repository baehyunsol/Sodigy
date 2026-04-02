use crate::{
    Assert,
    Bytecode,
    Label,
    Memory,
    Offset,
    Session,
    Value,
};
use sodigy_hir::EnumRepr;
use sodigy_mir::{Block, Callable, Expr, If, Match};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_parse::Field;

// It generates bytecodes that
//    1) evaluates the expr
//    2) moves the value to `dst`
pub fn lower_expr(
    expr: &Expr,
    session: &mut Session,
    bytecodes: &mut Vec<Bytecode>,
    dst: Memory,
    is_tail_call: bool,
) {
    match expr {
        Expr::Ident { id, dotfish } => {
            assert!(dotfish.is_none());
            let src = match session.ssa_map.get(&id.def_span) {
                Some(src) => Memory::SSA(*src),
                None => match &id.origin {
                    NameOrigin::Foreign { kind } | NameOrigin::Local { kind } => match kind {
                        NameKind::Let { is_top_level: true } => {
                            let value_inited = session.get_local_label();
                            bytecodes.push(Bytecode::InitOrJump {
                                def_span: id.def_span.clone(),
                                func: Label::Global(id.def_span.clone()),
                                label: value_inited.clone(),
                            });
                            bytecodes.push(Bytecode::Move {
                                src: Memory::Return,
                                dst: Memory::Global(id.def_span.clone()),
                            });
                            bytecodes.push(Bytecode::Label(value_inited.clone()));
                            Memory::Global(id.def_span.clone())
                        },
                        NameKind::Func => {
                            bytecodes.push(Bytecode::Const {
                                value: Value::FuncPointer {
                                    def_span: id.def_span.clone(),

                                    // `Session::link()` will fill this
                                    program_counter: None,
                                },
                                dst: dst.clone(),
                            });

                            if is_tail_call {
                                let return_ssa = session.move_to_ssa(&dst, bytecodes);
                                bytecodes.push(Bytecode::Return(return_ssa));
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
                    src: src.clone(),
                    dst: dst.clone(),
                });
            }

            if is_tail_call {
                let return_ssa = session.move_to_ssa(&dst, bytecodes);
                bytecodes.push(Bytecode::Return(return_ssa));
            }
        },
        Expr::Constant(c) => {
            let value = session.lower_constant(c);
            bytecodes.push(Bytecode::Const { value, dst: dst.clone() });

            if is_tail_call {
                let return_ssa = session.move_to_ssa(&dst, bytecodes);
                bytecodes.push(Bytecode::Return(return_ssa));
            }
        },
        Expr::If(If { cond, true_value, false_value, .. }) => {
            let eval_true_value = session.get_local_label();
            let return_expr = session.get_local_label();
            let cond_ssa = session.get_ssa();
            let true_ssa = session.get_ssa();
            let false_ssa = session.get_ssa();
            lower_expr(
                cond,
                session,
                bytecodes,
                Memory::SSA(cond_ssa),
                /* is_tail_call: */ false,
            );
            bytecodes.push(Bytecode::JumpIf {
                value: Memory::SSA(cond_ssa),
                label: eval_true_value.clone(),
            });
            lower_expr(false_value, session, bytecodes, Memory::SSA(false_ssa), is_tail_call);

            if !is_tail_call {
                bytecodes.push(Bytecode::Jump(return_expr.clone()));
            }

            bytecodes.push(Bytecode::Label(eval_true_value.clone()));
            lower_expr(true_value, session, bytecodes, Memory::SSA(true_ssa), is_tail_call);

            if !is_tail_call {
                bytecodes.push(Bytecode::Phi { pair: (true_ssa, false_ssa), dst });
            }

            bytecodes.push(Bytecode::Label(return_expr.clone()));
        },
        Expr::Match(Match { .. }) => unreachable!(),
        Expr::Block(Block { lets, asserts, value, .. }) => {
            for r#let in lets.iter() {
                let ssa_reg = session.get_ssa();
                session.ssa_map.insert(r#let.name_span.clone(), ssa_reg);
                let dst = Memory::SSA(ssa_reg);
                lower_expr(
                    &r#let.value,
                    session,
                    bytecodes,
                    dst.clone(),
                    /* is_tail_call: */ false,
                );
            }

            for assert in asserts.iter() {
                bytecodes.extend(Assert::from_mir(assert, session, /* is_top_level: */ false).bytecodes);
            }

            lower_expr(value, session, bytecodes, dst, is_tail_call);
        },
        Expr::Field { lhs, fields, dotfish } => {
            assert!(dotfish.last().unwrap().is_none());
            let ssa_reg = session.get_ssa();
            lower_expr(
                lhs,
                session,
                bytecodes,
                Memory::SSA(ssa_reg),
                /* is_tail_call: */ false,
            );
            let mut curr_ssa_reg = ssa_reg;

            for field in fields.iter() {
                let ssa_reg = session.get_ssa();

                match field {
                    Field::Index(i) => {
                        bytecodes.push(Bytecode::Move {
                            src: Memory::Heap {
                                ptr: Box::new(Memory::SSA(curr_ssa_reg)),
                                // NOTE: There are no negative index because post-mir already lowered them
                                offset: Offset::Static(*i as u32),
                            },
                            dst: Memory::SSA(ssa_reg),
                        });
                    },
                    _ => panic!("TODO: {field:?}"),
                }

                curr_ssa_reg = ssa_reg;
            }

            bytecodes.push(Bytecode::Move {
                src: Memory::SSA(curr_ssa_reg),
                dst: dst.clone(),
            });

            if is_tail_call {
                let return_ssa = session.move_to_ssa(&dst, bytecodes);
                bytecodes.push(Bytecode::Return(return_ssa));
            }
        },
        Expr::Call { func, args, .. } => {
            match func {
                Callable::Static { .. } | Callable::Dynamic(_) => {
                    let mut arg_ssa_regs = Vec::with_capacity(args.len());

                    for arg in args.iter() {
                        let ssa_reg = session.get_ssa();
                        arg_ssa_regs.push(ssa_reg);
                        lower_expr(
                            arg,
                            session,
                            bytecodes,
                            Memory::SSA(ssa_reg),
                            /* is_tail_call: */ false,
                        );
                    }

                    match func {
                        Callable::Static { def_span, .. } => match session.intrinsics.get(def_span) {
                            Some(intrinsic) => {
                                bytecodes.push(Bytecode::Intrinsic {
                                    intrinsic: *intrinsic,
                                    args: arg_ssa_regs,
                                    dst: dst.clone(),
                                });

                                if is_tail_call {
                                    let return_ssa = session.move_to_ssa(&dst, bytecodes);
                                    bytecodes.push(Bytecode::Return(return_ssa));
                                }
                            },
                            None => {
                                let func = Label::Global(def_span.clone());
                                bytecodes.push(Bytecode::Call {
                                    func,
                                    args: arg_ssa_regs,
                                    tail: is_tail_call,
                                });

                                if !is_tail_call {
                                    bytecodes.push(Bytecode::Move {
                                        src: Memory::Return,
                                        dst,
                                    });
                                }
                            },
                        },
                        Callable::Dynamic(f) => {
                            let func_ssa = session.get_ssa();
                            lower_expr(
                                f,
                                session,
                                bytecodes,
                                Memory::SSA(func_ssa),
                                /* is_tail_call: */ false,
                            );

                            bytecodes.push(Bytecode::CallDynamic {
                                func: Memory::SSA(func_ssa),
                                args: arg_ssa_regs,
                                tail: is_tail_call,
                            });

                            if !is_tail_call {
                                bytecodes.push(Bytecode::Move {
                                    src: Memory::SSA(func_ssa),
                                    dst,
                                });
                            }
                        },
                        _ => unreachable!(),
                    }
                },
                Callable::StructInit { .. } |
                Callable::TupleInit { .. } => {
                    bytecodes.push(Bytecode::InitTuple {
                        elements: args.len(),
                        dst: dst.clone(),
                    });

                    for (i, arg) in args.iter().enumerate() {
                        lower_expr(
                            arg,
                            session,
                            bytecodes,
                            Memory::Heap {
                                ptr: Box::new(dst.clone()),
                                offset: Offset::Static(i as u32),
                            },
                            /* is_tail_call: */ false,
                        );
                    }

                    if is_tail_call {
                        let return_ssa = session.move_to_ssa(&dst, bytecodes);
                        bytecodes.push(Bytecode::Return(return_ssa));
                    }
                },
                Callable::EnumInit { parent_def_span, variant_def_span, .. } => {
                    let enum_shape = session.global_context.enum_shapes.unwrap().get(parent_def_span).unwrap();
                    let variant_index = *enum_shape.variant_index.get(variant_def_span).unwrap();

                    match enum_shape.representation {
                        EnumRepr::Scalar => {
                            assert!(args.is_empty());
                            bytecodes.push(Bytecode::Const {
                                value: Value::Scalar(variant_index as u32),
                                dst: dst.clone(),
                            });
                        },
                        EnumRepr::Compound => {
                            bytecodes.push(Bytecode::InitTuple {
                                elements: args.len() + 1,
                                dst: dst.clone(),
                            });
                            bytecodes.push(Bytecode::Const {
                                value: Value::Scalar(variant_index as u32),
                                dst: Memory::Heap {
                                    ptr: Box::new(dst.clone()),
                                    offset: Offset::Static(variant_index as u32),
                                },
                            });

                            for (i, arg) in args.iter().enumerate() {
                                lower_expr(
                                    arg,
                                    session,
                                    bytecodes,
                                    Memory::Heap {
                                        ptr: Box::new(dst.clone()),
                                        offset: Offset::Static(i as u32 + 1),
                                    },
                                    /* is_tail_call: */ false,
                                );
                            }
                        },
                        EnumRepr::Niche => todo!(),
                    }

                    if is_tail_call {
                        let return_ssa = session.move_to_ssa(&dst, bytecodes);
                        bytecodes.push(Bytecode::Return(return_ssa));
                    }
                },
                Callable::ListInit { .. } => {
                    bytecodes.push(Bytecode::InitList {
                        elements: args.len(),
                        dst: dst.clone(),
                    });

                    for (i, arg) in args.iter().enumerate() {
                        lower_expr(
                            arg,
                            session,
                            bytecodes,
                            Memory::List {
                                ptr: Box::new(dst.clone()),
                                offset: Offset::Static(i as u32),
                            },
                            /* is_tail_call: */ false,
                        );
                    }

                    if is_tail_call {
                        let return_ssa = session.move_to_ssa(&dst, bytecodes);
                        bytecodes.push(Bytecode::Return(return_ssa));
                    }
                },
            }
        },
        _ => panic!("TODO: {expr:?}"),
    }
}
