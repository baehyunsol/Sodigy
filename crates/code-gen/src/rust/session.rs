use super::BasicBlocksInspection;
use sodigy_bytecode::SSA;
use std::collections::{HashMap, HashSet};

pub struct Session {
    // For `global_ssa` and `phi`, read the comments above `struct BasicBlocksInspection`.`
    pub global_ssa: HashSet<SSA>,
    pub phi: HashMap<SSA, (SSA, SSA)>,
}

impl Session {
    pub fn from_inspection(inspection: &BasicBlocksInspection) -> Self {
        Session {
            global_ssa: inspection.global_ssa.clone(),
            phi: inspection.phi.clone(),
        }
    }
}
