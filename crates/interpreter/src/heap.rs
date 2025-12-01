use sodigy_file::File;
use sodigy_lir::{DebugInfoKind, Value};
use sodigy_span::Span;

// mmm  rrr  d00  d01  d02  ...
//
// mmm: metadata of this block
//   - the first byte is whether this block is used or not (1 for used)
//   - the remaining 31 bytes is the size of this block (only counting `d` scalars)
// rrr: ref count of this block
// d00..: actual data
//
// pointer points to `mmm`, not `d00`.

const SMALL_BLOCK_SIZE: usize = 8;
const MEDIUM_BLOCK_SIZE: usize = 256;
const LARGE_BLOCK_SIZE: usize = 8192;

pub struct Heap {
    pub debug_info: Vec<(DebugInfoKind, u32)>,

    pub data: Vec<u32>,
    pub freelist_small: Vec<usize>,
    pub freelist_medium: Vec<usize>,
    pub freelist_large: Vec<usize>,
}

impl Heap {
    pub fn new() -> Heap {
        let mut data = vec![];
        let mut freelist_small = vec![];
        let mut freelist_medium = vec![];
        let mut freelist_large = vec![];

        for _ in 0..512 {
            freelist_small.push(data.len());
            data.push(SMALL_BLOCK_SIZE as u32);
            data.extend(vec![0; SMALL_BLOCK_SIZE + 1]);
        }

        for _ in 0..128 {
            freelist_medium.push(data.len());
            data.push(MEDIUM_BLOCK_SIZE as u32);
            data.extend(vec![0; MEDIUM_BLOCK_SIZE + 1]);
        }

        for _ in 0..32 {
            freelist_large.push(data.len());
            data.push(LARGE_BLOCK_SIZE as u32);
            data.extend(vec![0; LARGE_BLOCK_SIZE + 1]);
        }

        Heap {
            debug_info: vec![],
            data,
            freelist_small,
            freelist_medium,
            freelist_large,
        }
    }

    pub fn alloc_value(&mut self, value: &Value) -> u32 {
        match value {
            Value::Scalar(v) => *v,
            Value::Compound(vs) => {
                let ptr = self.alloc(vs.len()) as u32;

                for (i, v) in vs.iter().enumerate() {
                    let v_p = self.alloc_value(v);
                    self.data[ptr as usize + 2 + i] = v_p;
                }

                ptr
            },
            Value::Span(span) => match span {
                Span::Range { file: File::File { project, file }, start, end } => {
                    let ptr = self.alloc(4);

                    // TODO: any better representation?
                    self.data[ptr + 2] = *project;
                    self.data[ptr + 3] = *file;
                    self.data[ptr + 4] = *start as u32;
                    self.data[ptr + 5] = *end as u32;
                    ptr as u32
                },
                _ => todo!(),
            },
        }
    }

    // It implicitly `inc_rc` after allocating memory.
    pub fn alloc(&mut self, size: usize) -> usize {
        if size <= SMALL_BLOCK_SIZE {
            if let Some(ptr) = self.freelist_small.pop() {
                self.data[ptr] |= 0x8000_0000;
                self.data[ptr + 1] = 1;
                ptr
            }

            else if let Some(ptr) = self.freelist_medium.pop() {
                todo!()
            }

            else {
                todo!()
            }
        }

        else if size <= MEDIUM_BLOCK_SIZE {
            if let Some(ptr) = self.freelist_medium.pop() {
                self.data[ptr] |= 0x8000_0000;
                self.data[ptr + 1] = 1;
                ptr
            }

            else if let Some(ptr) = self.freelist_large.pop() {
                todo!()
            }

            else {
                todo!()
            }
        }

        else {
            todo!()
        }
    }

    // It does not `dec_rc` after freeing memory.
    pub fn free(&mut self, ptr: usize) {
        let size = self.data[ptr] & 0x7fff_ffff;
        self.data[ptr] = size;

        // TODO: If its adjacent block is free, concat them!

        if size <= SMALL_BLOCK_SIZE as u32 {
            self.freelist_small.push(ptr);
        }

        else if size <= MEDIUM_BLOCK_SIZE as u32 {
            self.freelist_medium.push(ptr);
        }

        else {
            self.freelist_large.push(ptr);
        }
    }

    pub fn inc_rc(&mut self, ptr: usize) {
        self.data[ptr + 1] += 1;
    }
}
