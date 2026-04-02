use sodigy_bytecode::{
    Bytecode,
    Executable,
    Label,
    Memory,
    Offset,
    Value,
};
use sodigy_mir::Intrinsic;
use sodigy_number::{
    BigInt,
    add_bi,
    div_bi,
    eq_bi,
    gt_bi,
    ilog2_ubi,
    lt_bi,
    mul_bi,
    neg_bi,
    rem_bi,
    shl_ubi,
    shr_ubi,
    sub_bi,
};

#[cfg(feature="debug-bytecode")]
mod debug;

mod heap;
mod stack;

pub use heap::Heap;
pub use stack::Stack;

pub fn interpret(executable: &Executable, label: usize) -> Result<(), ()> {
    let mut heap = Heap::new();
    let result = call(Stack::new(), &mut heap, executable, label);

    #[cfg(feature="debug-heap")] {
        heap.check_integrity();
    }

    match result {
        Ok(_) => Ok(()),

        // TODO: dump debug info!
        Err(()) => Err(()),
    }
}

fn call(
    mut stack: Stack,
    heap: &mut Heap,
    executable: &Executable,
    label: usize,
) -> Result<u32, ()> {
    let mut cursor = label;

    loop {
        #[cfg(feature="debug-bytecode")] {
            debug::debug(&stack, heap, &executable.bytecodes, cursor);
        }

        match &executable.bytecodes[cursor] {
            Bytecode::Const { value, dst } => {
                let value = heap.alloc_value(value);
                update(dst, value, &mut stack, heap);
            },
            Bytecode::Move { src, dst } => {
                let value = read(src, &stack, heap);
                update(dst, value, &mut stack, heap);
            },
            Bytecode::Phi { pair, dst } => {
                let value = match (stack.ssa.get(&pair.0), stack.ssa.get(&pair.1)) {
                    (Some(x), _) => *x,
                    (_, Some(y)) => *y,
                    _ => unreachable!(),
                };
                update(dst, value, &mut stack, heap);
            },
            Bytecode::Jump(label) => match label {
                Label::Flatten(i) => {
                    cursor = *i;
                    continue;
                },
                _ => unreachable!(),
            },
            Bytecode::Call { func, args, tail } => {
                let new_stack = Stack::from_args(args, &stack);
                let pc = match func {
                    Label::Flatten(i) => *i,
                    _ => unreachable!(),
                };

                if *tail {
                    stack = new_stack;
                    cursor = pc as usize;
                    continue;
                }

                else {
                    stack.r#return = call(new_stack, heap, executable, pc as usize)?;
                }
            },
            Bytecode::CallDynamic { func, args, tail } => {
                let new_stack = Stack::from_args(args, &stack);
                let pc = read(func, &stack, heap);

                if *tail {
                    stack = new_stack;
                    cursor = pc as usize;
                    continue;
                }

                else {
                    stack.r#return = call(new_stack, heap, executable, pc as usize)?;
                }
            },
            Bytecode::JumpIf { value, label } => {
                let value = read(value, &stack, heap);

                if value != 0 {
                    match label {
                        Label::Flatten(i) => {
                            cursor = *i;
                            continue;
                        },
                        _ => unreachable!(),
                    }
                }
            },
            Bytecode::InitOrJump { def_span, func, label } => {
                if heap.global_values.contains_key(def_span) {
                    match label {
                        Label::Flatten(i) => {
                            cursor = *i;
                            continue;
                        },
                        _ => unreachable!(),
                    }
                } else {
                    match func {
                        Label::Flatten(i) => {
                            stack.r#return = call(Stack::new(), heap, executable, *i)?;
                        },
                        _ => unreachable!(),
                    }
                }
            },
            Bytecode::Return(i) => {
                return Ok(*stack.ssa.get(i).unwrap());
            },
            Bytecode::Intrinsic { intrinsic, args, dst } => match intrinsic {
                Intrinsic::NegInt => {
                    let rhs_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let (rhs_neg, rhs) = inspect_int(&heap.data, rhs_ptr);
                    let (is_neg, nums) = neg_bi(rhs_neg, rhs);

                    let v = Value::Int(BigInt {
                        is_neg,
                        nums,
                    });
                    let ptr = heap.alloc_value(&v);
                    update(dst, ptr, &mut stack, heap);
                },
                Intrinsic::AddInt |
                Intrinsic::SubInt |
                Intrinsic::MulInt |
                Intrinsic::DivInt |
                Intrinsic::RemInt |
                Intrinsic::LtInt |
                Intrinsic::EqInt |
                Intrinsic::GtInt => {
                    let lhs_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let (lhs_neg, lhs) = inspect_int(&heap.data, lhs_ptr);

                    let rhs_ptr = *stack.ssa.get(&args[1]).unwrap() as usize;
                    let (rhs_neg, rhs) = inspect_int(&heap.data, rhs_ptr);

                    let result = match intrinsic {
                        Intrinsic::AddInt |
                        Intrinsic::SubInt |
                        Intrinsic::MulInt |
                        Intrinsic::DivInt |
                        Intrinsic::RemInt => {
                            let (is_neg, nums) = match intrinsic {
                                Intrinsic::AddInt => add_bi(lhs_neg, lhs, rhs_neg, rhs),
                                Intrinsic::SubInt => sub_bi(lhs_neg, lhs, rhs_neg, rhs),
                                Intrinsic::MulInt => mul_bi(lhs_neg, lhs, rhs_neg, rhs),
                                Intrinsic::DivInt => div_bi(lhs_neg, lhs, rhs_neg, rhs),
                                Intrinsic::RemInt => rem_bi(lhs_neg, lhs, rhs_neg, rhs),
                                _ => unreachable!(),
                            };
                            let v = Value::Int(BigInt {
                                is_neg,
                                nums,
                            });
                            let ptr = heap.alloc_value(&v);
                            ptr
                        },
                        Intrinsic::LtInt => if lt_bi(lhs_neg, lhs, rhs_neg, rhs) { 1 } else { 0 },
                        Intrinsic::EqInt => if eq_bi(lhs_neg, lhs, rhs_neg, rhs) { 1 } else { 0 },
                        Intrinsic::GtInt => if gt_bi(lhs_neg, lhs, rhs_neg, rhs) { 1 } else { 0 },
                        _ => unreachable!(),
                    };

                    update(dst, result, &mut stack, heap);
                },
                Intrinsic::ShrInt | Intrinsic::ShlInt => {
                    let lhs_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let (is_neg, lhs) = inspect_int(&heap.data, lhs_ptr);
                    let rhs = *stack.ssa.get(&args[1]).unwrap();

                    let nums = match intrinsic {
                        Intrinsic::ShrInt => shr_ubi(lhs, rhs),
                        Intrinsic::ShlInt => shl_ubi(lhs, rhs),
                        _ => unreachable!(),
                    };
                    let v = Value::Int(BigInt {
                        is_neg,
                        nums,
                    });
                    let result = heap.alloc_value(&v);
                    update(dst, result, &mut stack, heap);
                },
                Intrinsic::Ilog2Int => {
                    let lhs = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let (_, rhs) = inspect_int(&heap.data, lhs);
                    let result = ilog2_ubi(rhs);
                    update(dst, result, &mut stack, heap);
                },
                Intrinsic::LtScalar |
                Intrinsic::EqScalar |
                Intrinsic::GtScalar => {
                    let lhs = *stack.ssa.get(&args[0]).unwrap();
                    let rhs = *stack.ssa.get(&args[1]).unwrap();
                    let result = match intrinsic {
                        Intrinsic::LtScalar => lhs < rhs,
                        Intrinsic::EqScalar => lhs == rhs,
                        Intrinsic::GtScalar => lhs > rhs,
                        _ => unreachable!(),
                    };
                    update(dst, result as u32, &mut stack, heap);
                },
                Intrinsic::ScalarToInt => {
                    let lhs = *stack.ssa.get(&args[0]).unwrap();
                    let result = heap.alloc_int_from_u32(lhs);
                    update(dst, result, &mut stack, heap);
                },
                Intrinsic::IntToScalar => {
                    let lhs = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let (_, n) = inspect_int(&heap.data, lhs);
                    update(dst, n[0], &mut stack, heap);
                },
                Intrinsic::IndexList => {
                    let slice_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let index = *stack.ssa.get(&args[1]).unwrap() as usize;
                    let buffer_ptr = heap.data[slice_ptr] as usize;
                    let start = heap.data[slice_ptr + 1] as usize;
                    let result = heap.data[buffer_ptr + start + index + 1];
                    update(dst, result, &mut stack, heap);
                },
                Intrinsic::LenList => {
                    let slice_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let result = heap.data[slice_ptr + 2];
                    update(dst, result, &mut stack, heap);
                },
                Intrinsic::SliceList => {
                    let slice_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let slice_start = *stack.ssa.get(&args[1]).unwrap();
                    let slice_end = *stack.ssa.get(&args[2]).unwrap();
                    let buffer_ptr = heap.data[slice_ptr];
                    let start = heap.data[slice_ptr + 1];
                    let length = heap.data[slice_ptr + 2];

                    let new_slice_ptr = heap.alloc(3);
                    heap.data[new_slice_ptr] = buffer_ptr as u32;
                    heap.data[new_slice_ptr + 1] = start + slice_start;
                    heap.data[new_slice_ptr + 2] = slice_end - slice_start;
                    update(dst, new_slice_ptr as u32, &mut stack, heap);
                },
                Intrinsic::SliceRightList => {
                    let slice_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let slice_start = *stack.ssa.get(&args[1]).unwrap();
                    let buffer_ptr = heap.data[slice_ptr];
                    let start = heap.data[slice_ptr + 1];
                    let length = heap.data[slice_ptr + 2];

                    let new_slice_ptr = heap.alloc(3);
                    heap.data[new_slice_ptr] = buffer_ptr as u32;
                    heap.data[new_slice_ptr + 1] = start + slice_start;
                    heap.data[new_slice_ptr + 2] = length - slice_start;
                    update(dst, new_slice_ptr as u32, &mut stack, heap);
                },
                Intrinsic::AppendList => {
                    let slice_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let value = *stack.ssa.get(&args[1]).unwrap();

                    // TODO: I don't want to call `.to_vec()`, but the borrow checker forces me to do so.
                    let curr_list = inspect_list(&heap.data, slice_ptr).to_vec();

                    let new_buffer = heap.alloc(curr_list.len() + 2);
                    heap.data[new_buffer] = curr_list.len() as u32 + 1;
                    heap.data[new_buffer + curr_list.len() + 1] = value;

                    for (i, v) in curr_list.iter().enumerate() {
                        heap.data[new_buffer + i + 1] = *v;
                    }

                    let new_slice_ptr = heap.alloc(3);
                    heap.data[new_slice_ptr] = new_buffer as u32;
                    heap.data[new_slice_ptr + 1] = 0;
                    heap.data[new_slice_ptr + 2] = curr_list.len() as u32 + 1;

                    update(dst, new_slice_ptr as u32, &mut stack, heap);
                },
                Intrinsic::PrependList => todo!(),
                Intrinsic::Exit => {
                    // TODO: clean up stack and heap
                    return Ok(0);
                },
                Intrinsic::Panic => {
                    // TODO: clean up stack and heap
                    return Err(());
                },
                Intrinsic::Print | Intrinsic::EPrint => {
                    let chars_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                    let chars = inspect_list(&heap.data, chars_ptr);
                    let chars = chars.iter().map(
                        |ch| char::from_u32(*ch).expect("invalid char point")
                    ).collect::<Vec<_>>().into_iter().collect::<String>();

                    match intrinsic {
                        Intrinsic::Print => {
                            print!("{chars}");
                        },
                        Intrinsic::EPrint => {
                            eprint!("{chars}");
                        },
                        _ => unreachable!(),
                    }
                },
                Intrinsic::RandomInt => todo!(),
                Intrinsic::Nop => {
                    let v = *stack.ssa.get(&args[0]).unwrap();
                    update(dst, v, &mut stack, heap);
                },
            },
            Bytecode::InitTuple { elements, dst } => {
                let ptr = heap.alloc(*elements);
                update(dst, ptr as u32, &mut stack, heap);
            },
            Bytecode::InitList { elements, dst } => todo!(),
            Bytecode::PushDebugInfo { kind, src } => {
                let src = read(src, &stack, heap);
                heap.debug_info.push((*kind, src));
            },
            Bytecode::PopDebugInfo => {
                heap.debug_info.pop().unwrap();
            },
            b => panic!("TODO: {b:?}"),
        }

        cursor += 1;
    }
}

fn read(src: &Memory, stack: &Stack, heap: &Heap) -> u32 {
    match src {
        Memory::Return => stack.r#return,
        Memory::SSA(i) => *stack.ssa.get(i).unwrap(),
        Memory::Heap { ptr, offset } => {
            let ptr = read(ptr, stack, heap);
            let offset = match offset {
                Offset::Static(i) => *i,
                Offset::Dynamic(p) => read(p, stack, heap),
            };
            heap.data[(ptr + offset) as usize]
        },
        Memory::List { ptr, offset } => todo!(),
        Memory::Global(s) => *heap.global_values.get(s).expect("global should be initialized before used"),
    }
}

fn update(dst: &Memory, value: u32, stack: &mut Stack, heap: &mut Heap) {
    match dst {
        Memory::Return => {
            stack.r#return = value;
        },
        Memory::SSA(i) => {
            stack.ssa.insert(*i, value);
        },
        Memory::Heap { ptr, offset } => {
            let ptr = read(ptr, stack, heap);
            let offset = match offset {
                Offset::Static(i) => *i,
                Offset::Dynamic(p) => read(p, stack, heap),
            };

            heap.data[(ptr + offset) as usize] = value;
        },
        Memory::List { ptr, offset } => todo!(),
        Memory::Global(s) => {
            heap.global_values.insert(s.clone(), value);
        },
    }
}

fn inspect_int(heap: &[u32], ptr: usize) -> (bool, &[u32]) {
    let metadata = heap[ptr];
    let is_neg = metadata > 0x7fff_ffff;
    let length = metadata & 0x7fff_ffff;
    let nums = &heap[(ptr + 1)..(ptr + 1 + length as usize)];
    (is_neg, nums)
}

fn inspect_list(heap: &[u32], ptr: usize) -> &[u32] {
    let slice_ptr = heap[ptr] as usize;
    let start = heap[ptr + 1] as usize;
    let length = heap[ptr + 2] as usize;
    &heap[(slice_ptr + start + 1)..(slice_ptr + start + length + 1)]
}
