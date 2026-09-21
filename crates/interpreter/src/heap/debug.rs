use super::Heap;
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
    // It doesn't check whether there's a memory leak or not (ref_count).
    // We can't check memory leaks for now because we can't tell whether
    // it's a leaked memory or a static value.
    pub fn check_integrity(&self) {
        let mut cursor = 2;
        let freelist_3 = self.freelist_3.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_8 = self.freelist_8.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_38 = self.freelist_38.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_158 = self.freelist_158.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_638 = self.freelist_638.iter().map(|ptr| *ptr).collect::<HashSet<_>>();
        let freelist_large = self.freelist_large.iter().map(|ptr| *ptr).collect::<HashSet<_>>();

        loop {
            let header = self.data[cursor - 2];
            let block_size = header & 0x7fff_ffff;
            let is_used = header >= 0x8000_0000;
            assert!(block_size > 0);

            if is_used {
                assert_eq!(*self.heap_debug_info.allocations.get(&cursor).unwrap(), block_size);
            }

            else {
                match block_size {
                    3 => { assert!(freelist_3.contains(&cursor)); },
                    8 => { assert!(freelist_8.contains(&cursor)); },
                    38 => { assert!(freelist_38.contains(&cursor)); },
                    158 => { assert!(freelist_158.contains(&cursor)); },
                    638 => { assert!(freelist_638.contains(&cursor)); },
                    _ => {
                        assert!(block_size > 638);
                        assert!(freelist_large.contains(&cursor));
                    },
                }
            }

            cursor += block_size as usize + 2;

            if cursor >= self.data.len() {
                assert_eq!(cursor, self.data.len() + 2);
                break;
            }
        }
    }
}
