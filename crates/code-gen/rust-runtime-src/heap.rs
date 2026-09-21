// FIXME: This is a dummy implementation. I have to implement a real one...

pub struct Heap {
    pub data: Vec<u32>,
    pub global_values: std::collections::HashMap<u128, u32>,
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            data: vec![],
            global_values: std::collections::HashMap::new(),
        }
    }

    pub fn alloc(&mut self, size: usize) -> usize {
        todo!()
    }

    pub fn alloc_int(&mut self, is_neg: bool, nums: &[u32]) -> u32 {
        todo!()
    }

    pub fn inspect_int(&self, ptr: u32) -> (bool, &[u32]) {
        todo!()
    }
}
