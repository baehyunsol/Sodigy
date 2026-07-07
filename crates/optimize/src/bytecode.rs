use crate::OptimizeLevel;
use sodigy_bytecode::{Bytecode, Session};

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
}

fn optimize_local(bytecodes: &mut Vec<Bytecode>) {
    for bytecode in bytecodes.iter() {
        match bytecode {
            Bytecode::Const { value, dst: Memory::Heap { ptr, offset: Offset::Static(b) } } if let Memory::SSA(a) = ptr => {
                context.sroa.insert((*a, *b), SSAOrValue::Value(value.clone()));
            },
            Bytecode::Move { src: Memory::SSA(a), dst: Memory::SSA(b) } => {
                context.add_ssa_alias(a, b);
            },
            Bytecode::Move { src: Memory::SSA(c), dst: Memory::Heap { ptr, offset: Offset::Static(b) } } if let Memory::SSA(a) = ptr => {
                context.sroa.insert((*a, *b), SSAOrValue::SSA(*c));
            },
        }

        for ssa in bytecode.rhs_ssa() {
            match context.use_counts.entry(ssa) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += 1;
                },
                Entry::Vacant(e) => {
                    e.insert(1);
                },
            }
        }
    }

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
