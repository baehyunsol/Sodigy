use sodigy_lir::{
    Bytecode,
    Const,
    Label,
    Register,
};
use sodigy_mir::Intrinsic;
use sodigy_span::Span;
use std::collections::hash_map::{Entry, HashMap};

mod heap;

pub use heap::Heap;

const NULL: u32 = u32::MAX;

// It assumes that there's no error in `bytecode` and `init_label`.
pub fn interpret(
    bytecodes: &HashMap<u32, Vec<Bytecode>>,
    init_label: u32,
) -> Result<(), Vec<u32>> {  // If it panics, it returns the call stack
    let mut heap = Heap::new();
    let mut curr_label = init_label;
    let mut call_stack = vec![];
    let mut stacks: HashMap<Register, Vec<u32>> = HashMap::new();
    let mut consts: HashMap<Span, u32> = HashMap::new();
    let mut ret = NULL;

    'outer: loop {
        for bytecode in bytecodes.get(&curr_label).unwrap().iter() {
            match bytecode {
                Bytecode::Push { src, dst } => {
                    let ptr = match src {
                        Register::Local(_) |
                        Register::Call(_) => *stacks.get(src).unwrap().last().unwrap(),
                        Register::Return => ret,
                        Register::Const(c) => match consts.get(c) {
                            Some(v) => *v,
                            None => NULL,
                        },
                    };

                    if ptr != NULL {
                        heap.inc_rc(ptr);
                    }

                    match dst {
                        Register::Local(_) |
                        Register::Call(_) => match stacks.entry(*dst) {
                            Entry::Occupied(mut stack) => {
                                stack.get_mut().push(ptr);
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![ptr]);
                            },
                        },
                        Register::Return => {
                            ret = ptr;
                        },
                        Register::Const(c) => {
                            consts.insert(*c, ptr);
                        },
                    }
                },
                Bytecode::PushConst { value, dst } => {
                    let value = match value {
                        Const::Scalar(n) => *n,
                        Const::Number(n) => todo!(),
                        Const::String { s, binary } => todo!(),
                        Const::Compound(n) => todo!(),
                    };

                    match dst {
                        Register::Local(_) |
                        Register::Call(_) => match stacks.entry(*dst) {
                            Entry::Occupied(mut stack) => {
                                stack.get_mut().push(value);
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![value]);
                            },
                        },
                        Register::Return => {
                            ret = value;
                        },
                        Register::Const(c) => {
                            consts.insert(*c, value);
                        },
                    }
                },
                Bytecode::Pop(src) => {
                    let ptr = match src {
                        Register::Local(_) |
                        Register::Call(_) => stacks.get_mut(src).unwrap().pop().unwrap(),
                        Register::Return => ret,
                        Register::Const(_) => unreachable!(),
                    };

                    if ptr != NULL {
                        heap.dec_rc(ptr);
                    }
                },
                Bytecode::PushCallStack(label) => {
                    let Label::Static(n) = label else { unreachable!() };
                    call_stack.push(*n);
                },
                Bytecode::PopCallStack => {
                    call_stack.pop().unwrap();
                },
                Bytecode::Goto(label) => match label {
                    Label::Static(n) => {
                        curr_label = *n;
                        continue 'outer;
                    },
                    _ => unreachable!(),
                },
                Bytecode::Intrinsic(intrinsic) => match intrinsic {
                    Intrinsic::AddInt |
                    Intrinsic::SubInt |
                    Intrinsic::MulInt |
                    Intrinsic::DivInt |
                    Intrinsic::RemInt |
                    Intrinsic::LtInt |
                    Intrinsic::EqInt |
                    Intrinsic::GtInt => {
                        let (a, b) = (
                            *stacks.get(&Register::Call(0)).unwrap().last().unwrap(),
                            *stacks.get(&Register::Call(1)).unwrap().last().unwrap(),
                        );

                        match intrinsic {
                            _ => todo!(),
                        }
                    },
                    Intrinsic::Panic => {
                        call_stack.push(curr_label);
                        return Err(call_stack);
                    },
                    Intrinsic::Exit => {
                        return Ok(());
                    },
                    Intrinsic::Print => todo!(),
                    Intrinsic::EPrint => todo!(),
                },
                Bytecode::Label(_) => unreachable!(),
                Bytecode::Return => {
                    curr_label = *call_stack.last().unwrap();
                    continue 'outer;
                },
                Bytecode::JumpIf { value, label } => {
                    let value = match value {
                        Register::Local(_) |
                        Register::Call(_) => *stacks.get(value).unwrap().last().unwrap(),
                        Register::Return => ret,
                        Register::Const(c) => match consts.get(&c) {
                            Some(v) => *v,
                            None => NULL,
                        },
                    };

                    if is_true(value) {
                        let Label::Static(n) = label else { unreachable!() };
                        curr_label = *n;
                    }
                },
                Bytecode::JumpIfInit { reg, label } => {
                    let value = match reg {
                        Register::Local(_) |
                        Register::Call(_) => *stacks.get(reg).unwrap().last().unwrap(),
                        Register::Return => ret,
                        Register::Const(c) => match consts.get(&c) {
                            Some(v) => *v,
                            None => NULL,
                        },
                    };

                    if value != NULL {
                        let Label::Static(n) = label else { unreachable!() };
                        curr_label = *n;
                    }
                },
                _ => todo!(),
            }
        }
    }
}

fn is_true(ptr: u32) -> bool {
    if ptr == NULL {
        false
    }

    else {
        todo!()
    }
}
