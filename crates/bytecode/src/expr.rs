use crate::{
    Assert,
    Bytecode,
    InternedValue,
    Label,
    Memory,
    Offset,
    Session,
    SSA,
    Value,
};
use sodigy_hir::{EnumRepr, FuncShape};
use sodigy_mir::{Block, Callable, Expr, If, Match, Type, type_of};
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
                                def_span: id.def_span.hash(),
                                func: Label::Global(id.def_span.hash()),
                                label: value_inited.clone(),
                            });
                            bytecodes.push(Bytecode::Move {
                                src: Memory::Return,
                                dst: Memory::Global(id.def_span.hash()),
                            });
                            bytecodes.push(Bytecode::Label(value_inited.clone()));
                            Memory::Global(id.def_span.hash())
                        },
                        NameKind::Func => {
                            let func_pointer = Value::FuncPointer {
                                def_span: id.def_span.hash(),

                                // `crate::link::flatten(..)` will fill this
                                program_counter: None,
                            };

                            bytecodes.push(Bytecode::Const {
                                value: session.intern_value(&func_pointer),
                                dst: dst.clone(),
                                debug_info: if session.debug_info { Some(Box::new(id.span.clone())) } else { None },
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
            bytecodes.push(Bytecode::Const {
                value,
                dst: dst.clone(),
                debug_info: if session.debug_info { Some(Box::new(c.span())) } else { None },
            });

            if is_tail_call {
                let return_ssa = session.move_to_ssa(&dst, bytecodes);
                bytecodes.push(Bytecode::Return(return_ssa));
            }
        },
        Expr::If(If { if_span, cond, true_value, false_value, .. }) => {
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
                debug_info: if session.debug_info { Some(Box::new(if_span.clone())) } else { None },
            });
            lower_expr(false_value, session, bytecodes, Memory::SSA(false_ssa), is_tail_call);

            if !is_tail_call {
                bytecodes.push(Bytecode::Jump(return_expr.clone()));
            }

            bytecodes.push(Bytecode::Label(eval_true_value.clone()));
            lower_expr(true_value, session, bytecodes, Memory::SSA(true_ssa), is_tail_call);

            if !is_tail_call {
                bytecodes.push(Bytecode::Label(return_expr.clone()));
                bytecodes.push(Bytecode::Phi { pair: (true_ssa, false_ssa), dst });
            }
        },
        Expr::Match(Match { .. }) => unreachable!(),
        Expr::Block(Block { lets, asserts, dos, value, .. }) => {
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

            for r#do in dos.iter() {
                let null_ssa = session.get_ssa();
                let dst = Memory::SSA(null_ssa);
                lower_expr(
                    &r#do.value,
                    session,
                    bytecodes,
                    dst.clone(),
                    /* is_tail_call: */ false,
                );
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
                lower_field_read(curr_ssa_reg, field, ssa_reg, bytecodes);
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
        // Let's say we lower `` foo `x.y.z bar ``.
        // The result would be
        // ```
        // _t1 = foo;
        // _t2 = bar;
        // _t3 = _t1.x;
        // _t4 = _t3.y;
        // _t5 = Bytecode::Update { src: _t4, index: z, value: _t2 };
        // _t6 = Bytecode::Update { src: _t3, index: y, value: _t5 };
        // 
        // // this is the result
        // _t7 = Bytecode::Update { src: _t1, index: x, value: _t6 };
        // ```
        Expr::FieldUpdate { fields, lhs, rhs } => {
            let lhs_ssa = session.get_ssa();
            lower_expr(
                lhs,
                session,
                bytecodes,
                Memory::SSA(lhs_ssa),
                false,
            );

            let rhs_ssa = session.get_ssa();
            lower_expr(
                rhs,
                session,
                bytecodes,
                Memory::SSA(rhs_ssa),
                false,
            );

            let mut curr_ssa_reg = lhs_ssa;
            let mut sources = vec![lhs_ssa];

            for (i, field) in fields.iter().enumerate() {
                if i == fields.len() - 1 {
                    break;
                }

                let ssa_reg = session.get_ssa();
                lower_field_read(curr_ssa_reg, field, ssa_reg, bytecodes);
                sources.push(ssa_reg);
                curr_ssa_reg = ssa_reg;
            }

            for ((i, field), src) in fields.iter().enumerate().zip(sources).rev() {
                let ssa_reg = session.get_ssa();
                lower_field_update(
                    src,
                    field,
                    if i == fields.len() - 1 { rhs_ssa } else { curr_ssa_reg },
                    if i == 0 { dst.clone() } else { Memory::SSA(ssa_reg) },
                    bytecodes,
                );
                curr_ssa_reg = ssa_reg;
            }

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
                        Callable::Static { def_span, span } => match session.intrinsics.get(def_span) {
                            Some(intrinsic) => {
                                bytecodes.push(Bytecode::Intrinsic {
                                    intrinsic: *intrinsic,
                                    args: arg_ssa_regs,
                                    dst: dst.clone(),
                                    debug_info: if session.debug_info { Some(Box::new(span.clone())) } else { None },
                                });

                                if is_tail_call {
                                    let return_ssa = session.move_to_ssa(&dst, bytecodes);
                                    bytecodes.push(Bytecode::Return(return_ssa));
                                }
                            },
                            None => {
                                let func = Label::Global(def_span.hash());
                                let effect = match session.global_context.func_shapes.unwrap().get(def_span) {
                                    Some(FuncShape { effect, .. }) => Box::new(effect.clone()),
                                    _ => unreachable!(),
                                };

                                bytecodes.push(Bytecode::Call {
                                    func,
                                    args: arg_ssa_regs,
                                    dst: if is_tail_call { None } else { Some(dst) },
                                    debug_info: if session.debug_info { Some(Box::new(span.clone())) } else { None },
                                    effect,
                                });
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

                            let effect = match type_of(f, session.global_context.clone()) {
                                Some(Type::Func { effect, .. }) => Box::new(effect.clone()),
                                _ => unreachable!(),
                            };

                            bytecodes.push(Bytecode::CallDynamic {
                                func: Memory::SSA(func_ssa),
                                args: arg_ssa_regs,
                                dst: if is_tail_call { None } else { Some(dst) },
                                debug_info: if session.debug_info { Some(Box::new(f.error_span_wide())) } else { None },
                                effect,
                            });
                        },
                        _ => unreachable!(),
                    }
                },
                Callable::StructInit { .. } |
                Callable::TupleInit { .. } => {
                    let debug_info = match (session.debug_info, func) {
                        (true, Callable::TupleInit { group_span }) => Some(Box::new(group_span.clone())),
                        _ => None,
                    };

                    bytecodes.push(Bytecode::InitTuple {
                        elements: args.len(),
                        dst: dst.clone(),
                        debug_info,
                    });

                    let dst_ssa = if args.is_empty() {
                        None
                    } else {
                        Some(session.move_to_ssa(&dst, bytecodes))
                    };

                    for (i, arg) in args.iter().enumerate() {
                        lower_expr(
                            arg,
                            session,
                            bytecodes,
                            Memory::Heap {
                                ptr: dst_ssa.unwrap(),
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
                Callable::EnumInit { enum_def_span, variant_def_span, .. } => {
                    let enum_shape = session.global_context.enum_shapes.unwrap().get(enum_def_span).unwrap();
                    let variant_index = *enum_shape.variant_index.get(&variant_def_span.id().unwrap()).unwrap();

                    match enum_shape.representation {
                        EnumRepr::Scalar => {
                            assert!(args.is_empty());
                            bytecodes.push(Bytecode::Const {
                                value: InternedValue::Scalar(variant_index as u32),
                                dst: dst.clone(),
                                debug_info: None,
                            });
                        },
                        EnumRepr::Compound => {
                            let dst_ssa = session.move_to_ssa(&dst, bytecodes);
                            bytecodes.push(Bytecode::InitTuple {
                                elements: args.len() + 1,
                                dst: dst.clone(),
                                debug_info: None,
                            });
                            bytecodes.push(Bytecode::Const {
                                value: InternedValue::Scalar(variant_index as u32),
                                dst: Memory::Heap {
                                    ptr: dst_ssa,
                                    offset: Offset::Static(0),
                                },
                                debug_info: None,
                            });

                            for (i, arg) in args.iter().enumerate() {
                                lower_expr(
                                    arg,
                                    session,
                                    bytecodes,
                                    Memory::Heap {
                                        ptr: dst_ssa,
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
                Callable::ListInit { group_span } => {
                    bytecodes.push(Bytecode::InitList {
                        elements: args.len(),
                        dst: dst.clone(),
                        debug_info: if session.debug_info { Some(Box::new(group_span.clone())) } else { None },
                    });

                    let dst_ssa = if args.is_empty() {
                        None
                    } else {
                        Some(session.move_to_ssa(&dst, bytecodes))
                    };

                    for (i, arg) in args.iter().enumerate() {
                        lower_expr(
                            arg,
                            session,
                            bytecodes,
                            Memory::List {
                                ptr: dst_ssa.unwrap(),
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
        Expr::Macro { .. } => unreachable!(),
    }
}

fn lower_field_read(
    src: SSA,
    field: &Field,
    dst: SSA,
    bytecodes: &mut Vec<Bytecode>,
) {
    match field {
        Field::Index(i) => {
            bytecodes.push(Bytecode::Move {
                src: Memory::Heap {
                    ptr: src,

                    // NOTE: There are no negative index because post-mir already lowered them
                    offset: Offset::Static(*i as u32),
                },
                dst: Memory::SSA(dst),
            });
        },
        Field::EnumPayload { payload, .. } => {
            bytecodes.push(Bytecode::Move {
                src: Memory::Heap {
                    ptr: src,

                    // Without the niche optimization, an enum is a tuple, where the first element
                    // is the variant discriminant, and the other elements are the payload.
                    offset: Offset::Static(*payload as u32 + 1),
                },
                dst: Memory::SSA(dst),
            });
        },
        Field::SelfAsScalar => {
            if src != dst {
                bytecodes.push(Bytecode::Move {
                    src: Memory::SSA(src),
                    dst: Memory::SSA(dst),
                });
            }
        },
        _ => panic!("TODO: {field:?}"),
    }
}

fn lower_field_update(
    src: SSA,
    field: &Field,
    value: SSA,
    dst: Memory,
    bytecodes: &mut Vec<Bytecode>,
) {
    match field {
        Field::Index(i) => {
            bytecodes.push(Bytecode::Update {
                src,
                size: 100,  // TODO: I'm too lazy to calc the size, so I'm just giving a big enough number
                index: *i as usize,
                value,
                dst,
            });
        },
        Field::EnumPayload { payload, .. } => {
            bytecodes.push(Bytecode::Update {
                src,
                size: 100,  // TODO: I'm too lazy to calc the size, so I'm just giving a big enough number
                index: *payload as usize + 1,
                value,
                dst,
            });
        },
        _ => panic!("TODO: {field:?}"),
    }
}
