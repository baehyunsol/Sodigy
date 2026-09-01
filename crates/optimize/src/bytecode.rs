use crate::OptimizeLevel;
use sodigy_bytecode::{Bytecode, Label, Memory, Session, SSA, Value};
use sodigy_endec::Endec;
use sodigy_mir::Intrinsic;
use sodigy_string::hash;
use std::collections::hash_map::{Entry, HashMap};

#[cfg(test)]
mod tests;

struct LocalContext {
    // If there's `_5 = _7;`, we can replace all `_5` with `_7` and remove this bytecode.
    //
    // So, all `Bytecode::Move`s will be gone in the optimized bytecodes.
    ssa_alias: HashMap<SSA, SSA>,

    // `*(_2 + 1) = _3; _5 = *(_2 + 1);` -> `_5 = _3;`
    heap_ssa_alias: HashMap<(SSA, u32), SSA>,

    // Let's say we have `*_2 = X; *(_2 + 1) = Y;` and `_2` is not used.
    // Then we'll apply sroa to this: `_100 = X; _101 = Y;`.
    // This map will remember: `(2, 0) -> 100` and `(2, 1) -> 101`.
    sroa: HashMap<(SSA, u32), SSA>,

    use_counts: HashMap<SSA, usize>,

    // When `*(_2 + 1)` is used, indirect_use_count of `_2` is incremented!
    indirect_use_counts: HashMap<SSA, usize>,

    // Let's say we have `*_2 = _3; *(_2 + 1) = _5;`, and `*_2` is used again but
    // `*(_2 + 1)` is not used again. Then we can remove `*(_2 + 1) = _5;`.
    heap_use_counts: HashMap<(SSA, u32), usize>,

    // It's a `expr -> SSA` map. Let's say there are `_x = expr1;` and `_y = expr2;`. If `expr1` and `expr2` are the same,
    // this map will remember the fact and will later remove `_y = expr2;` and replace all `_y` with `_x`.
    common_expression: HashMap<ExprHash, Vec<SSA>>,
    free_ssa: SSA,
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
            free_ssa: SSA::from_u32(1000),
        }
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

    pub fn register_expression(&mut self, expr: ExprHash, ssa: SSA) {
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
        // If there's `_10 = _0; _20 = _10;`, we have to connect `_20` and `_0`.
        // There must be a better algorithm, but I'm not smart enough to figure that out...
        loop {
            let mut new_connection = HashMap::new();

            for (dst, src) in self.ssa_alias.iter() {
                if let Some(s2) = self.ssa_alias.get(src) {
                    if let Some(s3) = self.ssa_alias.get(s2) {
                        new_connection.insert(*dst, *s3);
                    }

                    else {
                        new_connection.insert(*dst, *s2);
                    }
                }
            }

            if new_connection.is_empty() {
                break;
            }

            for (dst, src) in new_connection.drain() {
                self.ssa_alias.insert(dst, src);
            }
        }

        // We have to sort this before inserting, so that the result is deterministic.
        let mut sroa_list = vec![];

        for (a, b) in self.heap_use_counts.keys() {
            if let Some(0) | None = self.use_counts.get(a) {
                sroa_list.push((*a, *b));
            }
        }

        sroa_list.sort_by_key(|(a, b)| ((a.to_u32() as u64) << 32) | *b as u64);
        let mut free_ssa = self.free_ssa;
        let mut sroa = HashMap::new();

        for s in sroa_list.into_iter() {
            sroa.insert(s, free_ssa);
            free_ssa.increment();
        }

        self.free_ssa = free_ssa;
        self.sroa = sroa;
    }
}

fn optimize_local(bytecodes: &mut Vec<Bytecode>) {
    let mut context = LocalContext::new();
    let mut max_ssa = SSA::from_u32(1000);

    for bytecode in bytecodes.iter() {
        match bytecode {
            Bytecode::Const { value, dst, .. } => {
                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_const(value), *a);
                    max_ssa = max_ssa.max(*a);
                }
            },
            Bytecode::Move { src, dst } => {
                context.count_use(src);

                if let Memory::SSA(a) = dst {
                    if let Memory::SSA(b) = src {
                        context.ssa_alias.insert(*a, *b);
                    }

                    max_ssa = max_ssa.max(*a);
                }

                if let Some((a, b)) = dst.get_heap_index() && let Memory::SSA(c) = src {
                    context.heap_ssa_alias.insert((a, b), *c);
                }
            },
            Bytecode::Phi { pair: (a, b), dst } => {
                context.count_use(&Memory::SSA(*a));
                context.count_use(&Memory::SSA(*b));

                if let Memory::SSA(c) = dst {
                    max_ssa = max_ssa.max(*c);
                }
            },
            Bytecode::Jump(_) => {},
            Bytecode::Call { func, args, dst, .. } => {
                for arg in args.iter() {
                    context.count_use(&Memory::SSA(*arg));
                }

                if let Some(Memory::SSA(a)) = dst {
                    context.register_expression(ExprHash::from_func_call(func, args), *a);
                    max_ssa = max_ssa.max(*a);
                }
            },
            Bytecode::CallDynamic { func, args, dst, .. } => {
                context.count_use(func);

                for arg in args.iter() {
                    context.count_use(&Memory::SSA(*arg));
                }

                if let Some(Memory::SSA(a)) = dst {
                    context.register_expression(ExprHash::from_dynamic_func_call(func, args), *a);
                    max_ssa = max_ssa.max(*a);
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
            Bytecode::Update { .. } => todo!(),
            Bytecode::Intrinsic { intrinsic, args, dst, .. } => {
                for arg in args.iter() {
                    context.count_use(&Memory::SSA(*arg));
                }

                if let Memory::SSA(a) = dst {
                    context.register_expression(ExprHash::from_intrinsic(*intrinsic, args), *a);
                    max_ssa = max_ssa.max(*a);
                }
            },
            Bytecode::InitTuple { dst, .. } => {
                if let Memory::SSA(a) = dst {
                    max_ssa = max_ssa.max(*a);
                }
            },
            Bytecode::InitList { dst, .. } => {
                if let Memory::SSA(a) = dst {
                    max_ssa = max_ssa.max(*a);
                }
            },
            Bytecode::PushDebugInfo { src, .. } => {
                context.count_use(src);
            },
            Bytecode::PopDebugInfo => {},
        }
    }

    context.free_ssa = max_ssa;
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
                        new_bytecodes.push(Bytecode::Move { src: Memory::SSA(*alias), dst: dst.clone() });
                        continue;
                    },
                    None => {},
                }
            }
        }

        if let Some(dst) = bytecode.get_dst() {
            if let Memory::SSA(a) = dst {
                match (context.use_counts.get(a), context.indirect_use_counts.get(a)) {
                    (Some(0) | None, Some(0) | None) if bytecode.is_observable() => {
                        continue;
                    },
                    (Some(1), _) if bytecode.is_observable() => {
                        // TODO: move this definition to the use point
                    },
                    _ => {},
                }
            }

            if let Some((a, b)) = dst.get_heap_index() {
                if let Some(0) | None = context.use_counts.get(&a) && bytecode.is_observable() {
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
    match level {
        OptimizeLevel::None => session,
        OptimizeLevel::Mild => {
            for func in session.funcs.iter_mut() {
                optimize_local(&mut func.bytecodes);
                optimize_local(&mut func.bytecodes);
            }

            session
        },
        OptimizeLevel::Extreme => {
            for func in session.funcs.iter_mut() {
                optimize_local(&mut func.bytecodes);
                optimize_local(&mut func.bytecodes);
                optimize_local(&mut func.bytecodes);
                optimize_local(&mut func.bytecodes);
                optimize_local(&mut func.bytecodes);
            }

            session
        },
    }
}
