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

    pub unused_ssa: HashSet<SSA>,
    pub writes_to_ret: bool,

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
    // Single basic block
    Single,

    // Three basic blocks where the first one terminates with JumpIf
    // and the other two are true-label and false-label.
    // It's purely for optimization.
    Triple,

    Multi,
}

pub fn inspect_basic_blocks(global_label: GlobalLabel, basic_blocks: &HashMap<LocalLabel, BasicBlock>) -> BasicBlocksInspection {
    let mut global_ssa = HashSet::new();
    let mut unused_ssa = HashSet::new();
    let mut writes_to_ret = false;
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
                match dst {
                    // It doesn't count `Memory::Heap` and `Memory::List` because they don't initialize SSA registers.
                    Memory::SSA(ssa) => {
                        ssa_write.insert(ssa);
                    },
                    Memory::Return => {
                        writes_to_ret = true;
                    },
                    _ => {},
                }
            }

            if let Bytecode::Phi { pair: (a, b), .. } = bytecode {
                phi.insert(*a, (*a, *b));
                phi.insert(*b, (*a, *b));
            }
        }

        for ssa in basic_block.terminator.used_ssa_indexes() {
            ssa_read.insert(ssa);
        }

        for ssa in ssa_read.iter() {
            if !ssa_write.contains(ssa) {
                global_ssa.insert(*ssa);
            }
        }

        for ssa in ssa_write.iter() {
            if !ssa_read.contains(ssa) {
                unused_ssa.insert(*ssa);
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
        unused_ssa,
        writes_to_ret,
        phi,
        shape,
        has_recursion,
    }
}
