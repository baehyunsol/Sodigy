use sodigy_lir::{
    Bytecode,
    Executable,
    Label,
    Memory,
};
use sodigy_mir::Intrinsic;
use sodigy_number::{
    BigInt,
    InternedNumber,
    InternedNumberValue,
    add_bi,
    div_bi,
    eq_bi,
    gt_bi,
    lt_bi,
    mul_bi,
    neg_bi,
    rem_bi,
    sub_bi,
};

mod heap;
mod stack;

pub use heap::Heap;
pub use stack::Stack;

pub fn interpret(executable: &Executable, label: usize) -> Result<(), ()> {
    let mut stack = Stack::with_capacity(65536);  // TODO: make the stack size configurable
    let mut heap = Heap::new();

    let result = execute(&mut stack, &mut heap, executable, label);
    
    #[cfg(feature="debug-heap")] {
        heap.check_integrity();
    }

    match result {
        Ok(()) => Ok(()),

        // TODO: dump debug info!
        Err(()) => Err(()),
    }
}

fn execute(
    stack: &mut Stack,
    heap: &mut Heap,
    executable: &Executable,
    label: usize,
) -> Result<(), ()> {
    let mut cursor = label;

    loop {
        #[cfg(feature="debug-bytecode")] {
            debug(stack, heap, &executable.bytecodes, cursor);
        }

        match &executable.bytecodes[cursor] {
            Bytecode::Const { value, dst } => {
                let value = heap.alloc_value(value);

                match dst {
                    Memory::Return => {
                        stack.r#return = value;
                    },
                    Memory::Stack(i) => {
                        *stack.stack.get_mut(stack.stack_pointer + i).expect("stack overflow") = value;
                    },
                    Memory::Global(s) => {
                        heap.global_values.insert(*s, value);
                    },
                }
            },
            Bytecode::Move { src, dst, inc_rc } => {
                let src = match src {
                    Memory::Return => stack.r#return,
                    Memory::Stack(i) => *stack.stack.get(stack.stack_pointer + i).expect("stack overflow"),
                    Memory::Global(s) => *heap.global_values.get(s).expect("global should be initialized before used"),
                };

                if *inc_rc {
                    heap.inc_rc(src as usize);
                }

                match dst {
                    Memory::Return => {
                        stack.r#return = src;
                    },
                    Memory::Stack(i) => {
                        *stack.stack.get_mut(stack.stack_pointer + i).expect("stack overflow") = src;
                    },
                    Memory::Global(s) => {
                        heap.global_values.insert(*s, src);
                    },
                }
            },
            Bytecode::IncStackPointer(n) => {
                stack.stack_pointer += n;
            },
            Bytecode::DecStackPointer(n) => {
                stack.stack_pointer -= n;
            },
            Bytecode::Jump(label) => match label {
                Label::Flatten(i) => {
                    cursor = *i;
                    continue;
                },
                _ => unreachable!(),
            },
            Bytecode::JumpIf { value, label } => {
                let value = match value {
                    Memory::Return => stack.r#return,
                    Memory::Stack(i) => *stack.stack.get(stack.stack_pointer + i).expect("stack overflow"),
                    Memory::Global(s) => *heap.global_values.get(s).expect("global should be initialized before used"),
                };

                if value == 1 {
                    match label {
                        Label::Flatten(i) => {
                            cursor = *i;
                            continue;
                        },
                        _ => unreachable!(),
                    }
                }
            },
            Bytecode::JumpIfUninit { def_span, label } => {
                if !heap.global_values.contains_key(def_span) {
                    match label {
                        Label::Flatten(i) => {
                            cursor = *i;
                            continue;
                        },
                        _ => unreachable!(),
                    }
                }
            },
            Bytecode::PushCallStack(label) => {
                let Label::Flatten(n) = label else { unreachable!() };
                stack.call_stack.push(*n);
            },
            Bytecode::PopCallStack => {
                stack.call_stack.pop().unwrap();
            },
            Bytecode::Return => {
                let dst = *stack.call_stack.last().unwrap();
                cursor = dst;
                continue;
            },
            Bytecode::Intrinsic { intrinsic, stack_offset, dst } => match intrinsic {
                Intrinsic::NegInt => {
                    let rhs_ptr = *stack.stack.get(stack.stack_pointer + *stack_offset).expect("stack overflow") as usize;
                    let (rhs_neg, rhs) = inspect_int(&heap.data[..], rhs_ptr);
                    let (is_neg, nums) = neg_bi(rhs_neg, rhs);

                    let v = InternedNumber {
                        value: InternedNumberValue::BigInt(BigInt {
                            is_neg,
                            nums,
                        }),
                        is_integer: true,
                    };

                    let ptr = heap.alloc_value(&(&v).into());

                    match dst {
                        Memory::Return => {
                            stack.r#return = ptr;
                        },
                        Memory::Stack(i) => {
                            *stack.stack.get_mut(stack.stack_pointer + i).expect("stack overflow") = ptr;
                        },
                        Memory::Global(s) => {
                            heap.global_values.insert(*s, ptr);
                        },
                    }
                },
                Intrinsic::AddInt |
                Intrinsic::SubInt |
                Intrinsic::MulInt |
                Intrinsic::DivInt |
                Intrinsic::RemInt |
                Intrinsic::LtInt |
                Intrinsic::EqInt |
                Intrinsic::GtInt => {
                    let lhs_ptr = *stack.stack.get(stack.stack_pointer + *stack_offset).expect("stack overflow") as usize;
                    let (lhs_neg, lhs) = inspect_int(&heap.data[..], lhs_ptr);

                    let rhs_ptr = *stack.stack.get(stack.stack_pointer + *stack_offset + 1).expect("stack overflow") as usize;
                    let (rhs_neg, rhs) = inspect_int(&heap.data[..], rhs_ptr);

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
                            let v = InternedNumber {
                                value: InternedNumberValue::BigInt(BigInt {
                                    is_neg,
                                    nums,
                                }),
                                is_integer: true,
                            };
                            let ptr = heap.alloc_value(&(&v).into());
                            ptr
                        },
                        Intrinsic::LtInt => if lt_bi(lhs_neg, lhs, rhs_neg, rhs) { 1 } else { 0 },
                        Intrinsic::EqInt => if eq_bi(lhs_neg, lhs, rhs_neg, rhs) { 1 } else { 0 },
                        Intrinsic::GtInt => if gt_bi(lhs_neg, lhs, rhs_neg, rhs) { 1 } else { 0 },
                        _ => unreachable!(),
                    };

                    match dst {
                        Memory::Return => {
                            stack.r#return = result;
                        },
                        Memory::Stack(i) => {
                            *stack.stack.get_mut(stack.stack_pointer + i).expect("stack overflow") = result;
                        },
                        Memory::Global(s) => {
                            heap.global_values.insert(*s, result);
                        },
                    }
                },
                Intrinsic::Exit => {
                    // TODO: clean up stack and heap
                    return Ok(());
                },
                Intrinsic::Panic => {
                    // TODO: clean up stack and heap
                    return Err(());
                },
                Intrinsic::Print | Intrinsic::EPrint => {
                    let chars_ptr = *stack.stack.get(stack.stack_pointer + *stack_offset).expect("stack overflow") as usize;
                    let chars = inspect_list(&heap.data[..], chars_ptr);
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
            },
            Bytecode::PushDebugInfo { kind, src } => {
                let src = match src {
                    Memory::Return => stack.r#return,
                    Memory::Stack(i) => stack.stack[stack.stack_pointer + i],
                    Memory::Global(s) => *heap.global_values.get(s).expect("global should be initialized before used"),
                };

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

#[cfg(feature="debug-bytecode")]
fn debug(
    stack: &Stack,
    heap: &Heap,
    bytecodes: &[Bytecode],
    cursor: usize,
) {
    println!("-------");
    println!("return: 0x{:08x}", stack.r#return);
    println!(
        "stack pointer: {:04x}\nstack: {}...",
        stack.stack_pointer,
        stack.stack.iter().skip(stack.stack_pointer).take(5).map(
            |v| format!("0x{v:08x}")
        ).collect::<Vec<_>>().join(", "),
    );
    println!(
        "call_stack: {:?}",
        if stack.call_stack.len() > 10 {
            &stack.call_stack[(stack.call_stack.len() - 10)..]
        } else {
            &stack.call_stack
        },
    );
    println!("- heap");

    for v in std::iter::once(&stack.r#return).chain(stack.stack.iter().skip(stack.stack_pointer).take(5)) {
        let heap = if *v as usize >= heap.data.len() {
            String::from("N/A")
        } else {
            let data = &heap.data[(*v as usize)..(*v as usize + 5).min(heap.data.len())];
            format!(
                "{}...",
                data.iter().map(
                    |n| format!("0x{n:08x}")
                ).collect::<Vec<_>>().join(", "),
            )
        };

        println!("0x{v:08x}: {heap}");
    }

    println!();

    for c in (cursor.max(2) - 2)..(cursor + 3).min(bytecodes.len()) {
        if c == cursor {
            println!("{} |", if cursor + 2 > 1000 { "       " } else { "     " });
        }

        println!(
            "{}{} | {:?}",
            if c == cursor { "->" } else { "  " },
            if cursor + 2 > 1000 { format!("{c:>5}") } else { format!("{c:>3}") },
            &bytecodes[c],
        );

        if c == cursor {
            println!("{} |", if cursor + 2 > 1000 { "       " } else { "     " });
        }
    }

    std::io::stdin().read_line(&mut String::new()).unwrap();
}

fn inspect_int(heap: &[u32], ptr: usize) -> (bool, &[u32]) {
    let metadata = heap[ptr + 2];
    let is_neg = metadata > 0x7fff_ffff;
    let length = metadata & 0x7fff_ffff;
    let nums = &heap[(ptr + 3)..(ptr + 3 + length as usize)];
    (is_neg, nums)
}

fn inspect_list(heap: &[u32], ptr: usize) -> &[u32] {
    let len_ptr = heap[ptr + 2];
    let (is_neg, length) = inspect_int(heap, len_ptr as usize);

    // TODO: should I do the runtime checks..??
    assert!(!is_neg);
    assert_eq!(length.len(), 1);

    &heap[(ptr + 3)..(ptr + 3 + length[0] as usize)]
}
