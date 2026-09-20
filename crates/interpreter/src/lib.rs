use sodigy_bytecode::{
    BasicBlock,
    Bytecode,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    ObjectFile,
    Terminator,
    Value,
};
use sodigy_code_gen::Profile;
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
use std::collections::HashMap;

// TODO: remove or fix this
#[cfg(feature="debug-bytecode")]
mod debug;

mod error;
mod heap;
mod stack;

pub use error::Error;
pub use heap::Heap;
pub use stack::Stack;

pub fn interpret(object_file: &ObjectFile, profile: Profile, intermediate_dir: &str) -> Result<(), Error> {
    let mut heap = Heap::new();

    match profile {
        Profile::Run => match object_file.main_entry {
            Some(label) => {
                let result = tail_call_loop(Stack::new(), &mut heap, object_file, label);

                #[cfg(feature="debug-heap")] {
                    heap.check_integrity();
                }

                match result {
                    CallResult::Return(_) |
                    CallResult::Exit(0) => Ok(()),
                    CallResult::TailCall { .. } => unreachable!(),
                    CallResult::Exit(n) => Err(Error::NonZeroExit(n.try_into().unwrap())),
                }
            },
            None => Err(Error::CannotFindEntry),
        },
        Profile::Test => {
            let mut heap = Heap::new();
            let mut ever_failed = false;

            for (name, label) in object_file.asserts.iter() {
                let result = tail_call_loop(Stack::new(), &mut heap, object_file, *label);

                #[cfg(feature = "debug-heap")] {
                    heap.check_integrity();
                }

                let fail = match result {
                    CallResult::Return(_) |
                    CallResult::Exit(0) => false,
                    CallResult::TailCall { .. } => unreachable!(),
                    CallResult::Exit(_) => {
                        // the heap might be corrupted
                        heap = Heap::new();
                        ever_failed = true;
                        true
                    },
                };

                println!("assertion `{name}`: {}", if fail { "fail" } else { "pass" });

                // FIXME
                // I want it to reset heap only when it panics.
                // But currently, there's no memory manager and it goes out of control so easily...
                // I have to remove this line when the memory manager is stable.
                heap = Heap::new();
            }

            if ever_failed {
                Err(Error::TestFail)
            } else {
                Ok(())
            }
        },
    }
}

fn tail_call_loop(
    mut stack: Stack,
    heap: &mut Heap,
    object_file: &ObjectFile,
    mut label: GlobalLabel,
) -> CallResult {
    loop {
        let basic_blocks = &object_file.code.get(&label).unwrap().basic_blocks;

        match call(stack, heap, object_file, basic_blocks) {
            CallResult::Return(n) => {
                return CallResult::Return(n);
            },
            CallResult::TailCall { func, stack: new_stack } => {
                stack = new_stack;
                label = func;
            },
            CallResult::Exit(n) => {
                return CallResult::Exit(n);
            },
        }
    }
}

enum CallResult {
    Return(u32),
    TailCall { func: GlobalLabel, stack: Stack },
    Exit(u8),
}

fn call(
    mut stack: Stack,
    heap: &mut Heap,
    object_file: &ObjectFile,
    basic_blocks: &HashMap<LocalLabel, BasicBlock>,
) -> CallResult {
    let mut curr_label = LocalLabel::start();

    loop {
        let curr_basic_block: &BasicBlock = basic_blocks.get(&curr_label).unwrap();

        for bytecode in curr_basic_block.code.iter() {
            match bytecode {
                Bytecode::Const { value, dst, debug_info: _ } => {
                    let value = match value {
                        InternedValue::Interned(h) => {
                            let value = object_file.data.get(h).unwrap();
                            heap.alloc_value(value)
                        },
                        InternedValue::Scalar(n) => *n,
                        InternedValue::FuncPointer(p) => heap.alloc_func_pointer(*p),
                    };
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
                Bytecode::Jump(_) => unreachable!(),
                Bytecode::Call { func, args, dst, debug_info: _, effect: _ } => {
                    let new_stack = Stack::from_args(args, &stack);

                    match dst {
                        Some(dst) => {
                            let value = match tail_call_loop(new_stack, heap, object_file, *func) {
                                CallResult::Return(n) => n,
                                CallResult::TailCall { .. } => unreachable!(),
                                CallResult::Exit(n) => {
                                    return CallResult::Exit(n);
                                },
                            };
                            update(dst, value, &mut stack, heap);
                        },
                        // tail call
                        None => unreachable!(),
                    }
                },
                Bytecode::CallDynamic { func, args, dst, debug_info: _, effect: _ } => {
                    let new_stack = Stack::from_args(args, &stack);
                    let func = stack.ssa.get(func).unwrap();
                    let func = heap.func_pointers_rev.get(func).unwrap();

                    match dst {
                        Some(dst) => {
                            let value = match tail_call_loop(new_stack, heap, object_file, GlobalLabel::new(*func)) {
                                CallResult::Return(n) => n,
                                CallResult::TailCall { .. } => unreachable!(),
                                CallResult::Exit(n) => {
                                    return CallResult::Exit(n);
                                },
                            };
                            update(dst, value, &mut stack, heap);
                        },
                        // tail call
                        None => unreachable!(),
                    }
                },
                Bytecode::JumpIf { .. } => unreachable!(),
                Bytecode::TryInitGlobal { .. } => unreachable!(),
                Bytecode::LoadGlobal { src, dst } => {
                    let src = heap.global_values.get(&src.span()).expect("global should be initialized before used");
                    stack.ssa.insert(*dst, *src);
                },
                Bytecode::StoreGlobal { src, dst } => {
                    let src = stack.ssa.get(src).unwrap();
                    heap.global_values.insert(dst.span(), *src);
                },
                Bytecode::Label(_) => unreachable!(),
                Bytecode::Return(_) => unreachable!(),
                Bytecode::Update { src, size, index, value, dst } => {
                    let ptr = *stack.ssa.get(src).unwrap() as usize;
                    let new_tuple = heap.alloc(*size);

                    for (i, v) in heap.data[ptr..(ptr + size)].to_vec().iter().enumerate() {
                        heap.data[new_tuple + i] = *v;
                    }

                    heap.data[new_tuple + index] = *stack.ssa.get(value).unwrap();
                    update(dst, new_tuple as u32, &mut stack, heap);
                },
                Bytecode::Intrinsic { intrinsic, args, dst, debug_info: _ } => match intrinsic {
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
                    Intrinsic::GtInt |
                    Intrinsic::BitAndInt |
                    Intrinsic::BitOrInt => {
                        let lhs_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                        let (lhs_neg, lhs) = inspect_int(&heap.data, lhs_ptr);

                        let rhs_ptr = *stack.ssa.get(&args[1]).unwrap() as usize;
                        let (rhs_neg, rhs) = inspect_int(&heap.data, rhs_ptr);

                        let result = match intrinsic {
                            Intrinsic::AddInt |
                            Intrinsic::SubInt |
                            Intrinsic::MulInt |
                            Intrinsic::DivInt |
                            Intrinsic::RemInt |
                            Intrinsic::BitAndInt |
                            Intrinsic::BitOrInt => {
                                let (is_neg, nums) = match intrinsic {
                                    Intrinsic::AddInt => add_bi(lhs_neg, lhs, rhs_neg, rhs),
                                    Intrinsic::SubInt => sub_bi(lhs_neg, lhs, rhs_neg, rhs),
                                    Intrinsic::MulInt => mul_bi(lhs_neg, lhs, rhs_neg, rhs),
                                    Intrinsic::DivInt => div_bi(lhs_neg, lhs, rhs_neg, rhs),
                                    Intrinsic::RemInt => rem_bi(lhs_neg, lhs, rhs_neg, rhs),
                                    _ => todo!(),
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
                    Intrinsic::AddScalar |
                    Intrinsic::SubScalar |
                    Intrinsic::MulScalar |
                    Intrinsic::DivScalar |
                    Intrinsic::RemScalar |
                    Intrinsic::BitAndScalar |
                    Intrinsic::BitOrScalar => {
                        let lhs = *stack.ssa.get(&args[0]).unwrap();
                        let rhs = *stack.ssa.get(&args[1]).unwrap();
                        let result = match intrinsic {
                            Intrinsic::AddScalar => lhs + rhs,
                            Intrinsic::SubScalar => lhs - rhs,
                            Intrinsic::MulScalar => lhs * rhs,
                            Intrinsic::DivScalar => lhs / rhs,
                            Intrinsic::RemScalar => lhs % rhs,
                            Intrinsic::BitAndScalar => lhs & rhs,
                            Intrinsic::BitOrScalar => lhs | rhs,
                            _ => unreachable!(),
                        };

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
                        let status_code = *stack.ssa.get(&args[0]).unwrap() as u8;

                        // TODO: clean up stack and heap
                        return CallResult::Exit(status_code);
                    },
                    Intrinsic::Print | Intrinsic::EPrint | Intrinsic::Debug => {
                        let chars_ptr = *stack.ssa.get(&args[0]).unwrap() as usize;
                        let chars = inspect_list(&heap.data, chars_ptr);
                        let chars = chars.iter().map(
                            |ch| char::from_u32(*ch).expect("invalid char point")
                        ).collect::<Vec<_>>().into_iter().collect::<String>();

                        match intrinsic {
                            Intrinsic::Print | Intrinsic::Debug => {
                                print!("{chars}");
                            },
                            Intrinsic::EPrint => {
                                eprint!("{chars}");
                            },
                            _ => unreachable!(),
                        }
                    },
                    Intrinsic::RandomInt => todo!(),
                    Intrinsic::Sleep => {
                        let n = *stack.ssa.get(&args[0]).unwrap() as usize;
                        let (_, ns) = inspect_int(&heap.data, n);
                        let n = match (ns.get(0), ns.get(1), ns.get(2)) {
                            (Some(n), None, _) => *n as u64,
                            (Some(a), Some(b), None) => *a as u64 | ((*b as u64) << 32),
                            _ => u64::MAX,  // who cares?
                        };

                        std::thread::sleep(std::time::Duration::from_millis(n));
                    },
                    Intrinsic::Nop0 => {},
                    Intrinsic::Nop1 => {
                        let v = *stack.ssa.get(&args[0]).unwrap();
                        update(dst, v, &mut stack, heap);
                    },
                },
                Bytecode::InitTuple { elements, dst, debug_info: _ } => {
                    let ptr = heap.alloc(*elements);
                    update(dst, ptr as u32, &mut stack, heap);
                },
                Bytecode::InitList { elements, dst, debug_info: _ } => {
                    let data_ptr = heap.alloc(*elements + 1);
                    heap.data[data_ptr] = *elements as u32;
                    let slice_ptr = heap.alloc(3);
                    heap.data[slice_ptr] = data_ptr as u32;
                    heap.data[slice_ptr + 1] = 0;
                    heap.data[slice_ptr + 2] = *elements as u32;
                    update(dst, slice_ptr as u32, &mut stack, heap);
                },
            }
        }

        match &curr_basic_block.terminator {
            Terminator::Jump(label) => {
                curr_label = *label;
            },
            Terminator::TailCall { func, args } => {
                return CallResult::TailCall { func: *func, stack: Stack::from_args(args, &stack) };
            },
            Terminator::TailCallDynamic { func, args } => todo!(),
            Terminator::JumpIf { value, t, f } => {
                let value = *stack.ssa.get(value).unwrap();

                if value != 0 {
                    curr_label = *t;
                } else {
                    curr_label = *f;
                }
            },
            Terminator::TryInitGlobal { global, label } => {
                if heap.global_values.get(&global.span()).is_none() {
                    match tail_call_loop(Stack::new(), heap, object_file, *global) {
                        CallResult::Return(_) => {},
                        CallResult::TailCall { .. } => unreachable!(),
                        CallResult::Exit(n) => {
                            return CallResult::Exit(n);
                        },
                    }

                    curr_label = *label;
                }
            },
            Terminator::Return(src) => {
                return CallResult::Return(*stack.ssa.get(src).unwrap());
            },
        }
    }
}

fn read(src: &Memory, stack: &Stack, heap: &Heap) -> u32 {
    match src {
        Memory::SSA(i) => *stack.ssa.get(i).unwrap(),
        Memory::Heap { ptr, offset } => {
            let ptr = *stack.ssa.get(ptr).unwrap();
            heap.data[(ptr + *offset) as usize]
        },
        Memory::List { ptr, offset } => {
            let ptr = *stack.ssa.get(ptr).unwrap() as usize;
            let data_ptr = heap.data[ptr];
            let start = heap.data[ptr + 1];
            heap.data[(data_ptr + start + *offset + 1) as usize]
        },
    }
}

fn update(dst: &Memory, value: u32, stack: &mut Stack, heap: &mut Heap) {
    match dst {
        Memory::SSA(i) => {
            stack.ssa.insert(*i, value);
        },
        Memory::Heap { ptr, offset } => {
            let ptr = *stack.ssa.get(ptr).unwrap();
            heap.data[(ptr + *offset) as usize] = value;
        },
        Memory::List { ptr, offset } => {
            let ptr = *stack.ssa.get(ptr).unwrap() as usize;
            let data_ptr = heap.data[ptr];
            let start = heap.data[ptr + 1];
            heap.data[(data_ptr + start + *offset + 1) as usize] = value;
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
