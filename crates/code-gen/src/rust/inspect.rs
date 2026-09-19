use sodigy_bytecode::{
    BasicBlock,
    Bytecode,
    GlobalLabel,
    LocalLabel,
    Memory,
    SSA,
    Terminator,
};
use std::collections::{HashMap, HashSet};

pub struct BasicBlocksInspection {
    // If SSA(100) is initialized in basic_block A and is used in basic_block B,
    // we have to add `let mut x100 = 0;` at the beginning of the code section, and
    // lvalue and rvalue of SSA(100) have to be `x100`.
    pub global_ssa: HashSet<SSA>,

    // If there's `Bytecode::Phi { pair: (100, 200), dst: SSA(300) }`, we have to
    // add `let mut p100200 = 0;` at the beginning of the code section, and lvalue
    // and rvalue of SSA(100) and SSA(200) have to be `p100200` and the bytecode
    // has to be lowered to `let x300 = p100200;`
    pub phi: HashMap<SSA, (SSA, SSA)>,

    pub shape: Shape,
    pub has_recursion: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Single,
    Triple,  // purely for optimization
    Multi,
}

pub fn inspect_basic_blocks(global_label: GlobalLabel, basic_blocks: &HashMap<LocalLabel, BasicBlock>) -> BasicBlocksInspection {
    let mut global_ssa = HashSet::new();
    let mut phi = HashMap::new();
    let mut has_recursion = false;

    for (label, basic_block) in basic_blocks.iter() {
        if let Terminator::TailCall { func, .. } = &basic_block.terminator && *func == global_label {
            has_recursion = true;
        }

        let mut ssa_read: HashSet<SSA> = HashSet::new();
        let mut ssa_write: HashSet<SSA> = HashSet::new();

        for bytecode in basic_block.code.iter() {
            for ssa in bytecode.used_ssa_indexes() {
                ssa_read.insert(ssa);
            }

            if let Some(dst) = bytecode.get_dst() {
                if let Memory::SSA(ssa) | Memory::Heap { ptr: ssa, .. } | Memory::List { ptr: ssa, .. } = dst {
                    ssa_write.insert(ssa);
                }
            }

            if let Bytecode::Phi { pair: (a, b), .. } = bytecode {
                phi.insert(*a, (*a, *b));
                phi.insert(*b, (*a, *b));
            }
        }

        for ssa in ssa_read.iter() {
            if !ssa_write.contains(ssa) {
                global_ssa.insert(*ssa);
            }
        }
    }

    let shape = match (basic_blocks.get(&LocalLabel::start()), basic_blocks.len()) {
        (_, 1) => Shape::Single,
        (Some(BasicBlock { terminator: Terminator::JumpIf { .. }, .. }), 3) => Shape::Triple,
        _ => Shape::Multi,
    };

    BasicBlocksInspection {
        global_ssa,
        phi,
        shape,
        has_recursion,
    }
}
