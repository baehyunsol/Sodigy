pub struct Heap {
    data: Vec<u32>,
}

impl Heap {
    pub fn new() -> Self {
        Heap { data: vec![] }
    }

    pub fn inc_rc(&mut self, ptr: u32) {
        self.data[ptr as usize] += 1;
    }

    pub fn dec_rc(&mut self, ptr: u32) {
        if self.data[ptr as usize] > 1 {
            self.data[ptr as usize] -= 1;
        }

        else {
            todo!()
        }
    }
}
