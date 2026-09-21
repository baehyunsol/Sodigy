use super::BasicBlocksInspection;
use sodigy_bytecode::SSA;
use std::collections::{HashMap, HashSet};

pub struct Session {
    // For `global_ssa` and `phi`, read the comments above `struct BasicBlocksInspection`.`
    pub global_ssa: HashSet<SSA>,
    pub unused_ssa: HashSet<SSA>,
    pub phi: HashMap<SSA, (SSA, SSA)>,

    // Whenever it initializes a list, it remembers the `data_ptr` of the list.
    pub data_ptrs: HashMap<SSA, u16>,
}

impl Session {
    pub fn from_inspection(inspection: &BasicBlocksInspection) -> Self {
        Session {
            global_ssa: inspection.global_ssa.clone(),
            unused_ssa: inspection.unused_ssa.clone(),
            phi: inspection.phi.clone(),
            data_ptrs: HashMap::new(),
        }
    }

    pub fn alloc_data_ptr_index(&mut self, slice_ptr: SSA) -> u16 {
        let i = self.data_ptrs.len() as u16;
        self.data_ptrs.insert(slice_ptr, i);
        i
    }
}
