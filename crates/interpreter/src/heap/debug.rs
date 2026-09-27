use super::Heap;
use std::collections::HashSet;

impl Heap {
    // TODO: return Err(_) instead of panicking
    pub fn check_integrity(&self) {
        let mut cursor = 2;
        let freelist_3 = self.freelist_3.iter().copied().collect::<HashSet<_>>();
        let freelist_8 = self.freelist_8.iter().copied().collect::<HashSet<_>>();
        let freelist_18 = self.freelist_18.iter().copied().collect::<HashSet<_>>();
        let freelist_38 = self.freelist_38.iter().copied().collect::<HashSet<_>>();
        let freelist_158 = self.freelist_158.iter().copied().collect::<HashSet<_>>();
        let freelist_638 = self.freelist_638.iter().copied().collect::<HashSet<_>>();
        let freelist_2558 = self.freelist_2558.iter().copied().collect::<HashSet<_>>();
        let freelist_large = self.freelist_large.iter().copied().collect::<HashSet<_>>();

        for (block_size, freelist) in [
            (3, &self.freelist_3),
            (8, &self.freelist_8),
            (18, &self.freelist_18),
            (38, &self.freelist_38),
            (158, &self.freelist_158),
            (638, &self.freelist_638),
            (2558, &self.freelist_2558),
        ] {
            for ptr in freelist.iter() {
                let header = self.data[ptr - 2];
                let real_size = header & 0x7fff_ffff;
                let is_used = header >= 0x8000_0000;
                let ref_count = self.data[ptr - 1];

                if real_size != block_size {
                    panic!("ptr {ptr:x} is in freelist_{block_size}, but its real size is {real_size}!!");
                }

                if is_used {
                    panic!("ptr {ptr:x} is in freelist_{block_size}, but is used!!");
                }

                if ref_count > 0 {
                    panic!("ptr {ptr:x} is in freelist_{block_size}, but its ref_count is {ref_count}!!");
                }
            }
        }

        loop {
            let header = self.data[cursor - 2];
            let block_size = header & 0x7fff_ffff;
            let is_used = header >= 0x8000_0000;
            let ref_count = self.data[cursor - 1];

            match block_size {
                3 => {
                    if is_used {
                        assert!(!freelist_3.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_3.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                8 => {
                    if is_used {
                        assert!(!freelist_8.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_8.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                18 => {
                    if is_used {
                        assert!(!freelist_18.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_18.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                38 => {
                    if is_used {
                        assert!(!freelist_38.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_38.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                158 => {
                    if is_used {
                        assert!(!freelist_158.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_158.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                638 => {
                    if is_used {
                        assert!(!freelist_638.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_638.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                2558 => {
                    if is_used {
                        assert!(!freelist_2558.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_2558.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                2559.. => {
                    if is_used {
                        assert!(!freelist_large.contains(&cursor));
                        assert!(ref_count > 0);
                    } else {
                        assert!(freelist_large.contains(&cursor));
                        assert_eq!(ref_count, 0);
                    }
                },
                _ => panic!("Block size {block_size} cannot fit in any freelist."),
            }

            cursor += block_size as usize + 2;

            if cursor >= self.data.len() {
                assert_eq!(cursor, self.data.len() + 2);
                break;
            }
        }
    }
}
