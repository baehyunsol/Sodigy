use super::{Heap, LARGE_BLOCK_SIZE, MEDIUM_BLOCK_SIZE};
use std::collections::{HashMap, HashSet};

pub struct HeapDebugInfo {
    pub allocations: HashMap</* ptr: */ usize, /* block_size: */ u32>,
}

impl HeapDebugInfo {
    pub fn new() -> HeapDebugInfo {
        HeapDebugInfo {
            allocations: HashMap::new(),
        }
    }
}

impl Heap {
    // It doesn't check whether there's a memory leak or not.
    // We can't check memory leaks for now because we can't tell whether
    // it's a leaked memory or a static value.
    pub fn check_integrity(&self) {
        let mut cursor = 0;
        let freelist_small = self.freelist_small.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_medium = self.freelist_medium.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_large = self.freelist_large.iter().map(|ptr| *ptr).collect::<HashSet<_>>();

        loop {
            let header = self.data[cursor];
            let block_size = header & 0x7fff_ffff;
            let is_used = header >= 0x8000_0000;
            assert!(block_size > 0);

            if is_used {
                assert_eq!(*self.heap_debug_info.allocations.get(&cursor).unwrap(), block_size);
            }

            else if block_size < MEDIUM_BLOCK_SIZE as u32 {
                assert!(freelist_small.contains(&cursor));
            }

            else if block_size < LARGE_BLOCK_SIZE as u32 {
                assert!(freelist_medium.contains(&cursor));
            }

            else {
                assert!(freelist_large.contains(&cursor));
            }

            cursor += block_size as usize + 2;

            if cursor >= self.data.len() {
                assert_eq!(cursor, self.data.len());
                break;
            }
        }
    }
}
