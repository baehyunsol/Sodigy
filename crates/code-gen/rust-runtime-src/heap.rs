// FIXME: This is a dummy implementation. I have to implement a real one...

pub struct Heap {
    pub data: Vec<u32>,
    pub global_values: std::collections::HashMap<u64, u32>,
}

impl Heap {
    pub fn new() -> Heap {
        todo!()
    }

    pub fn mut_list(&mut self, ptr: u32, offset: u32) -> &mut u32 {
        todo!()
    }

    pub fn init_tuple(&mut self, len: u32) -> u32 {
        todo!()
    }

    pub fn init_list(&mut self, len: u32) -> u32 {
        todo!()
    }

    pub fn read_list(&self, ptr: u32, offset: u32) -> u32 {
        todo!()
    }

    pub fn alloc_int(&mut self, is_neg: bool, nums: &[u32]) -> u32 {
        todo!()
    }

    pub fn inspect_int(&self, ptr: u32) -> (bool, &[u32]) {
        todo!()
    }
}
