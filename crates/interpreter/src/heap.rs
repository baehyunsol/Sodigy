use sodigy_bytecode::{DropType, Value};
use sodigy_span::SpanHash;
use std::collections::hash_map::{Entry, HashMap};

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
// pointer points to `d00`, not `hhh`.

pub struct Heap {
    pub data: Vec<u32>,

    // Global values are lazy-evaluated.
    // Global values are static: once initialized, it's alive until the end of the program.
    pub global_values: HashMap<SpanHash, u32>,

    // Func pointers are also lazy-evaluated.
    pub func_pointers: HashMap<SpanHash, u32>,
    pub func_pointers_rev: HashMap<u32, SpanHash>,

    // Each `usize` value is a pointer, where `self.data[pointer]` is the first data of a block.
    // `freelist_3` holds pointers to blocks that have 3 scalars for data (total 5 scalar, including the 2 scalar metadata).
    // Every block in a freelist has the same size.
    pub freelist_3: Vec<usize>,
    pub freelist_8: Vec<usize>,
    pub freelist_38: Vec<usize>,
    pub freelist_158: Vec<usize>,
    pub freelist_638: Vec<usize>,

    // for blocks bigger than 638 scalars
    pub freelist_large: Vec<usize>,

    #[cfg(feature="debug-heap")]
    pub heap_debug_info: HeapDebugInfo,
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            data: vec![],
            global_values: HashMap::new(),
            func_pointers: HashMap::new(),
            func_pointers_rev: HashMap::new(),
            freelist_3: vec![],
            freelist_8: vec![],
            freelist_38: vec![],
            freelist_158: vec![],
            freelist_638: vec![],
            freelist_large: vec![],

            #[cfg(feature="debug-heap")]
            heap_debug_info: HeapDebugInfo::new(),
        }
    }

    // It always creates 512 blocks. Linear grow is okay because `Vec<T>` grows exponentially.
    pub fn expand_3(&mut self) {
        let mut cursor = self.data.len() + 2;
        let mut new_buffer = vec![0; 2560];

        for i in 0..512 {
            new_buffer[i * 5] = 3;
            self.freelist_3.push(cursor);
            cursor += 5;
        }

        self.data.extend(new_buffer);
    }

    pub fn expand_8(&mut self) {
        let mut cursor = self.data.len() + 2;
        let mut new_buffer = vec![0; 2560];

        for i in 0..256 {
            new_buffer[i * 10] = 8;
            self.freelist_8.push(cursor);
            cursor += 10;
        }

        self.data.extend(new_buffer);
    }

    pub fn expand_38(&mut self) {
        let mut cursor = self.data.len() + 2;
        let mut new_buffer = vec![0; 2560];

        for i in 0..64 {
            new_buffer[i * 40] = 38;
            self.freelist_38.push(cursor);
            cursor += 40;
        }

        self.data.extend(new_buffer);
    }

    pub fn expand_158(&mut self) {
        let mut cursor = self.data.len() + 2;
        let mut new_buffer = vec![0; 2560];

        for i in 0..16 {
            new_buffer[i * 160] = 158;
            self.freelist_158.push(cursor);
            cursor += 160;
        }

        self.data.extend(new_buffer);
    }

    pub fn expand_638(&mut self) {
        let mut cursor = self.data.len() + 2;
        let mut new_buffer = vec![0; 2560];

        for i in 0..4 {
            new_buffer[i * 640] = 638;
            self.freelist_638.push(cursor);
            cursor += 640;
        }

        self.data.extend(new_buffer);
    }

    pub fn expand_large(&mut self, size: usize) {
        // TODO: assert size < 0x7fff_ffff
        //      -> runtime error vs panic?

        let mut new_buffer = vec![0; size + 2];
        new_buffer[0] = size as u32;
        self.freelist_large.push(self.data.len() + 2);
        self.data.extend(new_buffer);
    }

    pub fn alloc_func_pointer(&mut self, f: SpanHash) -> u32 {
        let new_index = self.func_pointers.len() as u32;

        match self.func_pointers.entry(f) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                e.insert(new_index);
                self.func_pointers_rev.insert(new_index, f);
                new_index
            },
        }
    }

    pub fn alloc_int_from_i32(&mut self, n: i32) -> u32 {
        let ptr = self.alloc(2) as u32;
        let metadata = if n < 0 { 0x8000_0001 } else { 1 };
        self.data[ptr as usize] = metadata;
        self.data[ptr as usize + 1] = n.abs() as u32;
        ptr
    }

    pub fn alloc_int_from_u32(&mut self, n: u32) -> u32 {
        let ptr = self.alloc(2) as u32;
        self.data[ptr as usize] = 1;
        self.data[ptr as usize + 1] = n;
        ptr
    }

    pub fn alloc_value(&mut self, value: &Value) -> u32 {
        match value {
            Value::Scalar(v) => *v,

            // heap:   hhh  rrr  d00  d01  d02 ...
            //                   ^
            //                   *-- pointer points to this
            //
            // `hhh` and `rrr` are memory allocator's metadata.
            // `d00` is the integer's metadata.
            //
            // The most significant bit of `d00` is whether the integer is negative or not.
            // If it's negative the most significant bit is 1.
            // The other bits is the length of the integer (how many scalar values).
            // `d01` and the remaining are the actual data.
            // `d01` is the least significant part of the integer.
            // So, the integer's value is `d01 + d02 * 4294967296 + d03 * 18446744073709551616 + ...`.
            //
            // If the integer is -5, it'd be represented like this:
            //
            // heap: [hhh, rrr, 0x8000_0001, 0x5]
            //                  ^
            //                  *-- the most significant bit is 1 because it's negative.
            //                  |
            //                  *-- pointer points to this
            Value::Int(n) => {
                let ptr = self.alloc(n.nums.len() + 1) as u32;
                let mut metadata = n.nums.len() as u32;

                if n.is_neg {
                    metadata |= 0x8000_0000;
                }

                self.data[ptr as usize] = metadata;

                for (i, n) in n.nums.iter().enumerate() {
                    self.data[ptr as usize + 1 + i] = *n;
                }

                ptr
            },

            // heap:   hhh  rrr  d00  d01  d02
            //                   ^
            //                   *-- pointer points here
            //
            // `hhh` and `rrr` are memory allocator's metadata.
            // `d00` is a pointer to the actual data
            // `d01` is a scalar value. It's the start index of the slice.
            // `d02` is a scalar value. It's the length of the slice.
            Value::List(vs) => {
                // Doesn't alloc for the data if the list is empty.
                let data_ptr = if vs.is_empty() {
                    0
                } else {
                    let data_ptr = self.alloc(vs.len() + 1);
                    self.data[data_ptr] = vs.len() as u32;

                    for (i, v) in vs.iter().enumerate() {
                        let v_p = self.alloc_value(v);
                        self.data[data_ptr + i + 1] = v_p;
                    }

                    data_ptr
                };

                let slice_ptr = self.alloc(3);
                self.data[slice_ptr] = data_ptr as u32;
                self.data[slice_ptr + 1] = 0;
                self.data[slice_ptr + 2] = vs.len() as u32;

                slice_ptr as u32
            },
            Value::Compound(vs) => {
                // TODO: don't alloc if `vs` is empty
                let ptr = self.alloc(vs.len()) as u32;

                for (i, v) in vs.iter().enumerate() {
                    let v_p = self.alloc_value(v);
                    self.data[ptr as usize + i] = v_p;
                }

                ptr
            },
            Value::FuncPointer(def_span) => todo!(),
        }
    }

    // `size` is of data, not block. A block has 1 scalar for header,
    // 1 scalar for ref_count, and scalars for data. If you call `alloc(8)`,
    // the returned block will have 10 scalars, where the first
    // 2 scalars are header and ref_count, and the remaining scalars are for data.
    pub fn alloc(&mut self, size: usize) -> usize {
        let result = match size {
            ..=3 => {
                if let Some(ptr) = self.freelist_3.pop() {
                    self.data[ptr - 2] = 0x8000_0003;
                    ptr
                } else if let Some(ptr) = self.freelist_8.pop() {
                    self.data[ptr - 2] = 0x8000_0003;
                    self.data[ptr + 3] = 0x0000_0003;
                    self.freelist_3.push(ptr + 5);
                    ptr
                } else {
                    self.expand_3();
                    self.alloc(size)
                }
            },
            ..=8 => {
                if let Some(ptr) = self.freelist_8.pop() {
                    self.data[ptr - 2] = 0x8000_0008;
                    ptr
                } else if let Some(ptr) = self.freelist_38.pop() {
                    self.data[ptr - 2] = 0x8000_0008;

                    self.data[ptr + 8] = 0x0000_0008;
                    self.freelist_8.push(ptr + 10);
                    self.data[ptr + 18] = 0x0000_0008;
                    self.freelist_8.push(ptr + 20);
                    self.data[ptr + 28] = 0x0000_0008;
                    self.freelist_8.push(ptr + 30);

                    ptr
                } else {
                    self.expand_8();
                    self.alloc(size)
                }
            },
            ..=38 => {
                if let Some(ptr) = self.freelist_38.pop() {
                    self.data[ptr - 2] = 0x8000_0026;
                    ptr
                } else if let Some(ptr) = self.freelist_158.pop() {
                    self.data[ptr - 2] = 0x8000_0026;

                    self.data[ptr + 38] = 0x0000_0026;
                    self.freelist_38.push(ptr + 40);
                    self.data[ptr + 78] = 0x0000_0026;
                    self.freelist_38.push(ptr + 80);
                    self.data[ptr + 118] = 0x0000_0026;
                    self.freelist_38.push(ptr + 120);

                    ptr
                } else {
                    self.expand_38();
                    self.alloc(size)
                }
            },
            ..=158 => {
                if let Some(ptr) = self.freelist_158.pop() {
                    self.data[ptr - 2] = 0x8000_009e;
                    ptr
                } else if let Some(ptr) = self.freelist_638.pop() {
                    self.data[ptr - 2] = 0x8000_009e;

                    self.data[ptr + 158] = 0x0000_009e;
                    self.freelist_158.push(ptr + 160);
                    self.data[ptr + 318] = 0x0000_009e;
                    self.freelist_158.push(ptr + 320);
                    self.data[ptr + 478] = 0x0000_009e;
                    self.freelist_158.push(ptr + 480);

                    ptr
                } else {
                    self.expand_158();
                    self.alloc(size)
                }
            },
            ..=638 => {
                if let Some(ptr) = self.freelist_638.pop() {
                    self.data[ptr - 2] = 0x8000_027e;
                    ptr
                } else {
                    self.expand_638();
                    self.alloc(size)
                }
            },
            _ => {
                if let Some(ptr) = self.freelist_large.pop() {
                    let block_size = self.data[ptr - 2] as usize;

                    if block_size >= size {
                        self.data[ptr - 2] |= 0x8000_0000;
                        ptr
                    } else {
                        self.freelist_large.push(ptr);
                        self.expand_large(size);
                        self.alloc(size)
                    }
                } else {
                    self.expand_large(size);
                    self.alloc(size)
                }
            },
        };

        #[cfg(feature="debug-heap")] {
            let block_size = self.data[result - 2] & 0x7fff_ffff;
            self.heap_debug_info.allocations.insert(result, block_size);
        }

        result
    }

    fn free(&mut self, ptr: usize) {
        let size = self.data[ptr - 2] & 0x7fff_ffff;
        self.data[ptr - 2] = size;

        #[cfg(feature="debug-heap")] {
            assert_eq!(self.heap_debug_info.allocations.remove(&ptr).unwrap(), size);
        }

        match size {
            3 => { self.freelist_3.push(ptr); },
            8 => { self.freelist_8.push(ptr); },
            38 => { self.freelist_38.push(ptr); },
            158 => { self.freelist_158.push(ptr); },
            638 => { self.freelist_638.push(ptr); },
            _ => { self.freelist_large.push(ptr); },
        }
    }

    pub fn inc_rc(&mut self, ptr: usize) {
        self.data[ptr - 1] += 1;
    }

    pub fn dec_rc(&mut self, ptr: usize) {
        self.data[ptr - 1] -= 1;
    }

    pub fn try_drop(&mut self, ptr: usize, drop: &DropType) {
        if self.data[ptr - 1] == 0 {
            // TODO: drop

            self.free(ptr);
        }
    }
}
