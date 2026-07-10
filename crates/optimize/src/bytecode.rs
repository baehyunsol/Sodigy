use crate::OptimizeLevel;
use sodigy_bytecode::{Bytecode, Label, Memory, Session, Value};
use sodigy_endec::Endec;
use sodigy_mir::Intrinsic;
use sodigy_string::hash;
use std::collections::hash_map::{Entry, HashMap};

struct LocalContext {
    // If there's `_5 = _7;`, we can replace all `_7` with `_5` and remove this bytecode.
    // It replaces the larger one with the smaller one.
    //
    // So, all `Bytecode::Move`s will be gone in the optimized bytecodes.
    ssa_alias: HashMap<u32, u32>,

    // `*(_2 + 1) = _3; _5 = *(_2 + 1);` -> `_5 = _3;`
    heap_ssa_alias: HashMap<(u32, u32), u32>,

    // Let's say we have `*_2 = X; *(_2 + 1) = Y;` and `_2` is not used.
    // Then we'll apply sroa to this: `_100 = X; _101 = Y;`.
    // This map will remember: `(2, 0) -> 100` and `(2, 1) -> 101`.
    sroa: HashMap<(u32, u32), u32>,

    use_counts: HashMap<u32, usize>,

    // When `*(_2 + 1)` is used, indirect_use_count of `_2` is incremented!
    indirect_use_counts: HashMap<u32, usize>,

    // Let's say we have `*_2 = _3; *(_2 + 1) = _5;`, and `*_2` is used again but
    // `*(_2 + 1)` is not used again. Then we can remove `*(_2 + 1) = _5;`.
    heap_use_counts: HashMap<(u32, u32), usize>,

    // It's a `expr -> SSA` map. Let's say there are `_x = expr1;` and `_y = expr2;`. If `expr1` and `expr2` are the same,
    // this map will remember the fact and will later remove `_y = expr2;` and replace all `_y` with `_x`.
    common_expression: HashMap<ExprHash, Vec<u32>>,
}

impl LocalContext {
    pub fn new() -> LocalContext {
        LocalContext {
            ssa_alias: HashMap::new(),
            heap_ssa_alias: HashMap::new(),
            sroa: HashMap::new(),
            use_counts: HashMap::new(),
            indirect_use_counts: HashMap::new(),
            heap_use_counts: HashMap::new(),
            common_expression: HashMap::new(),
        }
    }

    pub fn add_ssa_alias(&mut self, a: u32, b: u32) {
        let min = a.min(b);
        let min = *self.ssa_alias.get(&a).unwrap_or(&min).min(self.ssa_alias.get(&b).unwrap_or(&min));
        self.ssa_alias.insert(a, min);
        self.ssa_alias.insert(b, min);
    }

    pub fn count_use(&mut self, memory: &Memory) {
        if let Memory::SSA(ssa) = memory {
            match self.use_counts.entry(*ssa) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += 1;
                },
                Entry::Vacant(e) => {
                    e.insert(1);
                },
            }
        }

        if let Some((a, b)) = memory.get_heap_index() {
            match self.heap_use_counts.entry((a, b)) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += 1;
                },
                Entry::Vacant(e) => {
                    e.insert(1);
                },
            }

            match self.indirect_use_counts.entry(a) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += 1;
                },
                Entry::Vacant(e) => {
                    e.insert(1);
                },
            }
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

    pub fn finalize(&mut self) {
        let mut sroa = HashMap::new();

        for (a, b) in self.heap_use_counts.keys() {
            if let Some(0) | None = self.use_counts.get(a) {
                sroa.insert((*a, *b), todo!());
            }
        }

        self.sroa = sroa;
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ExprHash(u128);

impl ExprHash {
    pub fn from_const(c: &Value) -> ExprHash {
        let mut encoded = vec![0];
        c.encode_impl(&mut encoded);
        ExprHash(hash(&encoded))
    }

    pub fn from_func_call(f: &Label, args: &[u32]) -> ExprHash {
        let mut encoded = vec![1];
        f.encode_impl(&mut encoded);

        for arg in args.iter() {
            arg.encode_impl(&mut encoded);
        }

        ExprHash(hash(&encoded))
    }

    pub fn from_dynamic_func_call(f: &Memory, args: &[u32]) -> ExprHash {
        let mut encoded = vec![2];
        f.encode_impl(&mut encoded);

        for arg in args.iter() {
            arg.encode_impl(&mut encoded);
        }

        ExprHash(hash(&encoded))
    }

    pub fn from_intrinsic(f: Intrinsic, args: &[u32]) -> ExprHash {
        let mut encoded = vec![3];
        f.encode_impl(&mut encoded);

        for arg in args.iter() {
            arg.encode_impl(&mut encoded);
        }

        ExprHash(hash(&encoded))
    }
}

fn optimize_local(bytecodes: &mut Vec<Bytecode>) {
    let mut context = LocalContext::new();

    for bytecode in bytecodes.iter() {
        match bytecode {
            Bytecode::Const { value, dst, .. } => {
                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_const(value), *a);
                }
            },
            Bytecode::Move { src, dst } => {
                context.count_use(src);

                if let Memory::SSA(a) = dst && let Memory::SSA(b) = src {
                    context.add_ssa_alias(*a, *b);
                }

                if let Some((a, b)) = dst.get_heap_index() && let Memory::SSA(c) = src {
                    context.heap_ssa_alias.insert((a, b), *c);
                }
            },
            Bytecode::Phi { pair: (a, b), .. } => {
                context.count_use(&Memory::SSA(*a));
                context.count_use(&Memory::SSA(*b));
            },
            Bytecode::Jump(_) => {},
            Bytecode::Call { func, args, dst, .. } => {
                for arg in args.iter() {
                    context.count_use(&Memory::SSA(*arg));
                }

                if let Some(Memory::SSA(a)) = dst {
                    context.register_expression(ExprHash::from_func_call(func, args), *a);
                }
            },
            Bytecode::CallDynamic { func, args, dst, .. } => {
                context.count_use(func);

                for arg in args.iter() {
                    context.count_use(&Memory::SSA(*arg));
                }

                if let Some(Memory::SSA(a)) = dst {
                    context.register_expression(ExprHash::from_dynamic_func_call(func, args), *a);
                }
            },
            Bytecode::JumpIf { value, .. } => {
                context.count_use(value);
            },
            Bytecode::InitOrJump { .. } => {},
            Bytecode::Label(_) => {},
            Bytecode::Return(a) => {
                context.count_use(&Memory::SSA(*a));
            },
            Bytecode::Intrinsic { intrinsic, args, dst, .. } => {
                for arg in args.iter() {
                    context.count_use(&Memory::SSA(*arg));
                }

                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_intrinsic(*intrinsic, args), *a);
                }
            },
            Bytecode::InitTuple { .. } => {},
            Bytecode::InitList { .. } => {},
            Bytecode::PushDebugInfo { src, .. } => {
                context.count_use(src);
            },
            Bytecode::PopDebugInfo => {},
        }
    }

    context.finalize();

    let mut new_bytecodes: Vec<Bytecode> = Vec::with_capacity(bytecodes.len());

    for mut bytecode in bytecodes.drain(..) {
        if let Bytecode::Move { src, dst } = &bytecode {
            if let Memory::SSA(_) = src && let Memory::SSA(_) = dst {
                // `ssa_alias` must cover all the aliases.
                continue;
            }

            if let Some((a, b)) = src.get_heap_index() {
                match context.heap_ssa_alias.get(&(a, b)) {
                    Some(c) => {
                        let alias = context.ssa_alias.get(c).unwrap_or(c);
                        new_bytecodes.push(Bytecode::Move { src: src.clone(), dst: Memory::SSA(*alias) });
                        continue;
                    },
                    None => {},
                }
            }
        }

        if let Some(dst) = bytecode.get_dst() {
            if let Memory::SSA(a) = dst {
                match context.use_counts.get(a) {
                    Some(0) | None => {
                        continue;
                    },
                    Some(1) => {
                        // TODO: move this definition to the use point
                    },
                    _ => {},
                }
            }

            if let Some((a, b)) = dst.get_heap_index() {
                if let Some(0) | None = context.use_counts.get(&a) {
                    continue;
                }

                if let Some(alias) = context.sroa.get(&(a, b)) {
                    bytecode.set_dst(Memory::SSA(*alias));
                }
            }
        }

        bytecode.apply_ssa_alias(&context.ssa_alias, &context.heap_ssa_alias);
        new_bytecodes.push(bytecode);
    }

    *bytecodes = new_bytecodes;
}

pub fn optimize_bytecode<'hir, 'mir>(mut session: Session<'hir, 'mir>, level: OptimizeLevel) -> Session<'hir, 'mir> {
    if level == OptimizeLevel::None {
        return session;
    }

    for func in session.funcs.iter_mut() {
        optimize_local(&mut func.bytecodes);
    }

    session
}
