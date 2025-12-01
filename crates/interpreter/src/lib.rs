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

    match execute(&mut stack, &mut heap, executable, label) {
        Ok(()) => Ok(()),

        // dump debug info!
        Err(()) => todo!(),
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
        // debug(stack, heap, &executable.bytecodes, cursor);

        match &executable.bytecodes[cursor] {
            Bytecode::Const { value, dst } => {
                let value = heap.alloc_value(value);

                match dst {
                    Memory::Return => {
                        stack.r#return = value;
                    },
                    Memory::Stack(i) => {
                        stack.stack[stack.stack_pointer + i] = value;
                    },
                    Memory::Global(s) => todo!(),
                }
            },
            Bytecode::Move { src, dst, inc_rc } => {
                let src = match src {
                    Memory::Return => stack.r#return,
                    Memory::Stack(i) => stack.stack[stack.stack_pointer + i],
                    Memory::Global(s) => todo!(),
                };

                if *inc_rc {
                    heap.inc_rc(src as usize);
                }

                match dst {
                    Memory::Return => {
                        stack.r#return = src;
                    },
                    Memory::Stack(i) => {
                        stack.stack[stack.stack_pointer + i] = src;
                    },
                    Memory::Global(s) => todo!(),
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
                    Memory::Stack(i) => stack.stack[stack.stack_pointer + i],
                    Memory::Global(s) => todo!(),
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
            },
            Bytecode::Intrinsic { intrinsic, stack_offset, dst } => match intrinsic {
                Intrinsic::AddInt |
                Intrinsic::SubInt |
                Intrinsic::MulInt |
                Intrinsic::DivInt |
                Intrinsic::RemInt |
                Intrinsic::LtInt |
                Intrinsic::EqInt |
                Intrinsic::GtInt => {
                    let lhs_ptr = stack.stack[stack.stack_pointer + *stack_offset] as usize;
                    let lhs_meta = heap.data[lhs_ptr + 2];
                    let lhs_sign = lhs_meta > 0x7fff_ffff;
                    let lhs_len = lhs_meta & 0x7fff_ffff;
                    let lhs = &heap.data[(lhs_ptr + 3)..(lhs_ptr + 3 + lhs_len as usize)];

                    let rhs_ptr = stack.stack[stack.stack_pointer + *stack_offset + 1] as usize;
                    let rhs_meta = heap.data[rhs_ptr + 2];
                    let rhs_sign = rhs_meta > 0x7fff_ffff;
                    let rhs_len = rhs_meta & 0x7fff_ffff;
                    let rhs = &heap.data[(rhs_ptr + 3)..(rhs_ptr + 3 + rhs_len as usize)];

                    let result = match intrinsic {
                        Intrinsic::AddInt |
                        Intrinsic::SubInt |
                        Intrinsic::MulInt |
                        Intrinsic::DivInt |
                        Intrinsic::RemInt => {
                            let (is_neg, nums) = match intrinsic {
                                Intrinsic::AddInt => add_bi(lhs_sign, lhs, rhs_sign, rhs),
                                Intrinsic::SubInt => sub_bi(lhs_sign, lhs, rhs_sign, rhs),
                                Intrinsic::MulInt => mul_bi(lhs_sign, lhs, rhs_sign, rhs),
                                Intrinsic::DivInt => div_bi(lhs_sign, lhs, rhs_sign, rhs),
                                Intrinsic::RemInt => rem_bi(lhs_sign, lhs, rhs_sign, rhs),
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
                        Intrinsic::LtInt => if lt_bi(lhs_sign, lhs, rhs_sign, rhs) { 1 } else { 0 },
                        Intrinsic::EqInt => if eq_bi(lhs_sign, lhs, rhs_sign, rhs) { 1 } else { 0 },
                        Intrinsic::GtInt => if gt_bi(lhs_sign, lhs, rhs_sign, rhs) { 1 } else { 0 },
                        _ => unreachable!(),
                    };

                    match dst {
                        Memory::Return => {
                            stack.r#return = result;
                        },
                        Memory::Stack(i) => {
                            stack.stack[stack.stack_pointer + i] = result;
                        },
                        Memory::Global(s) => todo!(),
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
                _ => todo!(),
            },
            Bytecode::PushDebugInfo { kind, src } => {
                let src = match src {
                    Memory::Return => stack.r#return,
                    Memory::Stack(i) => stack.stack[stack.stack_pointer + i],
                    Memory::Global(s) => todo!(),
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

fn debug(
    stack: &Stack,
    heap: &Heap,
    bytecodes: &[Bytecode],
    cursor: usize,
) {
    println!("-------");
    println!("return: 0x{:08x}", stack.r#return);
    println!(
        "stack: {}...",
        stack.stack.iter().skip(stack.stack_pointer).take(5).map(
            |v| format!("0x{v:08x}")
        ).collect::<Vec<_>>().join(", "),
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
