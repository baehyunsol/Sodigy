use sodigy_bytecode::{DebugInfoKind, Value};
use sodigy_file::File;
use sodigy_span::Span;
use std::collections::HashMap;

#[cfg(feature="debug-heap")]
mod debug;

#[cfg(feature="debug-heap")]
use debug::HeapDebugInfo;

// hhh  rrr  d00  d01  d02  ...
//
// hhh: header of this block
//   - the first bit is whether this block is used or not (1 for used)
//   - the remaining 31 bits is the size of this block (only counting `d` scalars)
// rrr: ref count of this block
// d00..: actual data
//
// pointer points to `hhh`, not `d00`.

// You can change this constants to fine-tune performance.
// But the heap implementation assumes something and you have to follow this conditions:
// 1. SMALL_BLOCK_SIZE is at least 4.
// 2. MEDIUM_BLOCK_SIZE is at least as big as 8 times SMALL_BLOCK_SIZE.
// 3. LARGE_BLOCK_SIZE is at least as big as 8 times MEDIUM_BLOCK_SIZE.
// 4. LARGE_BLOCK_SIZE is smaller than 0x8000_0000.
const SMALL_BLOCK_SIZE: usize = 8;
const MEDIUM_BLOCK_SIZE: usize = 256;
const LARGE_BLOCK_SIZE: usize = 8192;

pub struct Heap {
    pub data: Vec<u32>,
    pub debug_info: Vec<(DebugInfoKind, u32)>,

    // Global values are lazy-evaluated.
    // Global values are static: once initialized, it's alive until the end of the program.
    pub global_values: HashMap<Span, u32>,

    // Blocks in freelist_small are at least as big as SMALL_BLOCK_SIZE (can be bigger).
    // Each `usize` value is a pointer, where `self.data[pointer]` is a header of a block.
    pub freelist_small: Vec<usize>,
    pub freelist_medium: Vec<usize>,
    pub freelist_large: Vec<usize>,

    #[cfg(feature="debug-heap")]
    pub heap_debug_info: HeapDebugInfo,
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            debug_info: vec![],
            global_values: HashMap::new(),
            data: vec![],
            freelist_small: vec![],
            freelist_medium: vec![],
            freelist_large: vec![],

            #[cfg(feature="debug-heap")]
            heap_debug_info: HeapDebugInfo::new(),
        }
    }

    pub fn expand(&mut self, s1: usize, s2: usize, s3: usize) {
        for _ in 0..s1 {
            self.freelist_small.push(self.data.len());
            self.data.push(SMALL_BLOCK_SIZE as u32);
            self.data.extend(vec![0; SMALL_BLOCK_SIZE + 1]);
        }

        for _ in 0..s2 {
            self.freelist_medium.push(self.data.len());
            self.data.push(MEDIUM_BLOCK_SIZE as u32);
            self.data.extend(vec![0; MEDIUM_BLOCK_SIZE + 1]);
        }

        for _ in 0..s3 {
            self.freelist_large.push(self.data.len());
            self.data.push(LARGE_BLOCK_SIZE as u32);
            self.data.extend(vec![0; LARGE_BLOCK_SIZE + 1]);
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
            Value::FuncPointer { program_counter, .. } => program_counter.unwrap() as u32,
            Value::Span(span) => match span {
                Span::Range { file: File::File { project, file }, start, end } |
                Span::Derived { file: File::File { project, file }, start, end, .. } => {
                    let ptr = self.alloc(4);

                    // TODO: any better representation?
                    self.data[ptr + 2] = *project;
                    self.data[ptr + 3] = *file;
                    self.data[ptr + 4] = *start as u32;
                    self.data[ptr + 5] = *end as u32;
                    ptr as u32
                },
                Span::Range { file: File::Std(id), start, end } |
                Span::Derived { file: File::Std(id), start, end, .. } => {
                    let ptr = self.alloc(4);

                    // TODO: any better representation?
                    self.data[ptr + 2] = u32::MAX;
                    self.data[ptr + 3] = *id as u32;
                    self.data[ptr + 4] = *start as u32;
                    self.data[ptr + 5] = *end as u32;
                    ptr as u32
                },
                _ => panic!("TODO: {span:?}"),
            },
        }
    }

    // It implicitly `inc_rc` after allocating memory.
    pub fn alloc(&mut self, size: usize) -> usize {
        let result = if size <= SMALL_BLOCK_SIZE {
            if let Some(ptr) = self.freelist_small.pop() {
                self.data[ptr] |= 0x8000_0000;
                self.data[ptr + 1] = 1;
                ptr
            }

            else if let Some(ptr) = self.freelist_medium.pop() {
                // this block is too big. I'll just use the quarter of this block.
                self.divide_block(ptr);

                self.data[ptr] |= 0x8000_0000;
                self.data[ptr + 1] = 1;
                ptr
            }

            else {
                // TODO: make it grow exponentially??
                self.expand(512, 0, 0);
                self.alloc(size)
            }
        }

        else if size <= MEDIUM_BLOCK_SIZE {
            if let Some(ptr) = self.freelist_medium.pop() {
                let block_size = self.data[ptr];

                if block_size > (size as u32 + 2) * 4 {
                    self.divide_block(ptr);
                }

                self.data[ptr] |= 0x8000_0000;
                self.data[ptr + 1] = 1;
                ptr
            }

            else if let Some(ptr) = self.freelist_large.pop() {
                todo!()
            }

            else {
                // TODO: make it grow exponentially??
                self.expand(0, 128, 0);
                self.alloc(size)
            }
        }

        else {
            todo!()
        };

        #[cfg(feature="debug-heap")] {
            let block_size = self.data[result] & 0x7fff_ffff;
            self.heap_debug_info.allocations.insert(result, block_size);
        }

        result
    }

    // It does not `dec_rc` after freeing memory.
    pub fn free(&mut self, ptr: usize) {
        let size = self.data[ptr] & 0x7fff_ffff;
        self.data[ptr] = size;

        #[cfg(feature="debug-heap")] {
            assert_eq!(self.heap_debug_info.allocations.remove(&ptr).unwrap(), size);
        }

        // TODO: If its adjacent block is free, concat them!

        if size < MEDIUM_BLOCK_SIZE as u32 {
            self.freelist_small.push(ptr);
        }

        else if size < LARGE_BLOCK_SIZE as u32 {
            self.freelist_medium.push(ptr);
        }

        else {
            self.freelist_large.push(ptr);
        }
    }

    pub fn inc_rc(&mut self, ptr: usize) {
        self.data[ptr + 1] += 1;
    }

    // `ptr` must be a header of an unused block.
    // It divides the block into 4.
    pub fn divide_block(&mut self, ptr: usize) {
        let original_size = self.data[ptr];
        let new_size = original_size >> 2;
        self.data[ptr] = new_size;

        for (header_ptr, block_size) in [
            (ptr + new_size as usize + 2, new_size),
            (ptr + new_size as usize * 2 + 4, new_size),
            (ptr + new_size as usize * 3 + 6, original_size - new_size * 3 - 6),
        ] {
            self.data[header_ptr] = block_size;

            if block_size < MEDIUM_BLOCK_SIZE as u32 {
                self.freelist_small.push(header_ptr);
            }

            else if block_size < LARGE_BLOCK_SIZE as u32 {
                self.freelist_medium.push(header_ptr);
            }

            else {
                self.freelist_large.push(header_ptr);
            }
        }
    }
}
