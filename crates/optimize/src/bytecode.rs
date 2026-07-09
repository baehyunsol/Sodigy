use crate::OptimizeLevel;
use sodigy_bytecode::{Bytecode, Memory, Session, Value};
use std::collections::hash_map::{Entry, HashMap};

struct LocalContext {
    // If there's `_5 = _7;`, we can replace all `_7` with `_5` and remove this bytecode.
    // It replaces the larger one with the smaller one.
    //
    // So, all `Bytecode::Move`s will be gone in the optimized bytecodes.
    ssa_alias: HashMap<u32, u32>,

    // `*(_2 + 1) = _3; _5 = *(_2 + 1);` -> `_5 = _3;`
    sroa: HashMap<(u32, u32), SSAOrValue>,

    use_counts: HashMap<u32, usize>,

    // Let's say we have `*_2 = _3; *(_2 + 1) = _5;`, and `*_2` is used again but
    // `*(_2 + 1)` is not used again. Then we can remove `*(_2 + 1) = _5;`.
    sroa_use_counts: HashMap<(u32, u32), usize>,

    // It's a `expr -> SSA` map. Let's say there are `_x = expr1;` and `_y = expr2;`. If `expr1` and `expr2` are the same,
    // this map will remember the fact and will later remove `_y = expr2;` and replace all `_y` with `_x`.
    common_expression: HashMap<ExprHash, Vec<u32>>,
}

impl LocalContext {
    pub fn new() -> LocalContext {
        LocalContext {
            ssa_alias: HashMap::new(),
            sroa: HashMap::new(),
            use_counts: HashMap::new(),
            sroa_use_counts: HashMap::new(),
            common_expression: HashMap::new(),
        }
    }

    pub fn count_use(&mut self, ssa: u32) {
        match self.use_counts.entry(ssa) {
            Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
            },
            Entry::Vacant(e) => {
                e.insert(1);
            },
        }
    }

    pub fn count_sroa_use(&mut self, ssa: u32, offset: u32) {
        match self.sroa_use_counts.entry((ssa, offset)) {
            Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
            },
            Entry::Vacant(e) => {
                e.insert(1);
            },
        }
    }

    pub fn register_expression(&mut self, expr: ExprHash, ssa: u32) {
        match self.common_expression.entry(expr) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(ssa);
            },
            Entry::Vacant(e) => {
                e.insert(vec![ssa]);
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ExprHash(u128);

enum SSAOrValue {
    SSA(u32),
    Value(Value),
}

fn optimize_local(bytecodes: &mut Vec<Bytecode>) {
    let mut context = LocalContext::new();

    for (i, bytecode) in bytecodes.iter().enumerate() {
        match bytecode {
            Bytecode::Const { value, dst, .. } => {
                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_const(value), *a);
                }

                if let Some((a, b)) = dst.get_sroa() {
                    context.sroa.insert((a, b), SSAOrValue::Value(value.clone()));
                }
            },
            Bytecode::Move { src, dst } => {
                if let Memory::SSA(b) = src {
                    context.count_use(*b);

                    if let Memory::SSA(a) = dst {
                        context.add_ssa_alias(*a, *b);
                    }
                }

                if let Some((a, b)) = src.get_sroa() {
                    context.count_sroa_use(a, b);
                }

                if let Some((a, b)) = dst.get_sroa() && let Memory::SSA(c) = src {
                    context.sroa.insert((a, b), SSAOrValue::SSA(*c));
                }
            },
            Bytecode::Phi { pair: (a, b), .. } => {
                context.count_use(*a);
                context.count_use(*b);
            },
            Bytecode::Jump(_) => {},
            Bytecode::Call { func, args, .. } => {
                for arg in args.iter() {
                    context.count_use(*arg);
                }

                if let Some(Bytecode::Move { src: Memory::Return, dst: Memory::SSA(a) }) = bytecodes.get(i + 1) {
                    context.register_expression(ExprHash::from_func_call(*func, args), *a);
                }
            },
            Bytecode::CallDynamic { func, args, .. } => {
                if let Memory::SSA(a) = func {
                    context.count_use(*a);
                }

                if let Some((a, b)) = func.get_sroa() {
                    context.count_sroa_use(a, b);
                }

                for arg in args.iter() {
                    context.count_use(*arg);
                }

                if let Some(Bytecode::Move { src: Memory::Return, dst: Memory::SSA(a) }) = bytecodes.get(i + 1) {
                    context.register_expression(ExprHash::from_dynamic_func_call(func, args), *a);
                }
            },
            Bytecode::JumpIf { value, .. } => {
                if let Memory::SSA(a) = value {
                    context.count_use(*a);
                }

                if let Some((a, b)) = value.get_sroa() {
                    context.count_sroa_use(a, b);
                }
            },
            Bytecode::InitOrJump { .. } => {},
            Bytecode::Label(_) => {},
            Bytecode::Return(a) => {
                context.count_use(*a);
            },
            Bytecode::Intrinsic { intrinsic, args, dst, .. } => {
                for arg in args.iter() {
                    context.count_use(*arg);
                }

                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_intrinsic(*intrinsic, args), *a);
                }
            },
            Bytecode::InitTuple { elements, dst, .. } => {
                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_init_tuple(*elements), *a);
                }
            },
            Bytecode::InitList { elements, dst, .. } => {
                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_init_list(*elements), *a);
                }
            },
            Bytecode::PushDebugInfo { src, .. } => {
                if let Memory::SSA(b) = src {
                    context.count_use(*b);
                }

                if let Some((a, b)) = src.get_sroa() {
                    context.count_sroa_use(a, b);
                }
            },
            Bytecode::PopDebugInfo => {},
        }
    }

    context.init();

    todo!()
}

pub fn optimize_bytecode<'hir, 'mir>(mut session: Session<'hir, 'mir>, level: OptimizeLevel) -> Session<'hir, 'mir> {
    // if level == OptimizeLevel::None {
    //     return session;
    // }

    for func in session.funcs.iter_mut() {
        optimize_local(&mut func.bytecodes);
    }

    session
}
