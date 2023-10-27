use crate::{DUMMY_FILE_HASH, read_bytes, FileHash};
use crate::err::FileError;
use sodigy_test::sodigy_assert_eq;
use std::sync::Mutex;

const FILE_CACHE_SIZE: usize = 32;
const SIZE_LIMIT: usize = 64 * 1024 * 1024;
static mut CACHE_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct FileCache {
    data: [(FileHash, Vec<u8>); FILE_CACHE_SIZE],

    // it's incremented everytime a file is read
    // when a file has to be removed, a file with the smallest count is removed
    count: [usize; FILE_CACHE_SIZE],

    // points the most recently removed file's index
    cursor: usize,

    // sum of data's len
    total_size: usize,
}

impl FileCache {
    pub fn new() -> Self {
        FileCache {
            // [(DUMMY_FILE_HASH), vec![]]
            data: Default::default(),
            count: [0; FILE_CACHE_SIZE],
            cursor: 0,
            total_size: 0,
        }
    }

    // TODO: lifetime of `self` and `[u8]` are different,
    // but the compiler doesn't know that
    pub fn get(&mut self, hash: FileHash) -> Option<&[u8]> {
        sodigy_assert_eq!(
            self.data.iter().map(
                |(_, file)| file.len()
            ).sum::<usize>(),
            self.total_size,
        );

        for i in 0..FILE_CACHE_SIZE {
            if self.data[i].0 == hash {
                self.count[i] += 1;
                return self.data.get(i).map(|(_, data)| data as &[u8]);
            }
        }

        None
    }

    pub fn insert(&mut self, hash: FileHash, path: &str) -> Result<(), FileError> {
        loop {
            if self.count[self.cursor] == 0 {
                match read_bytes(path) {
                    Ok(f) => unsafe {
                        let lock = CACHE_LOCK.lock();
                        self.total_size -= self.data[self.cursor].1.len();
                        self.total_size += f.len();

                        self.data[self.cursor] = (hash, f);
                        self.count[self.cursor] = 1;
                        self.cursor += 1;

                        if self.total_size > SIZE_LIMIT {
                            self.total_size -= self.data[self.cursor].1.len();
                            self.data[self.cursor] = (DUMMY_FILE_HASH, vec![]);
                            self.count[self.cursor] = 0;
                        }

                        drop(lock);

                        return Ok(());
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            else {
                self.count[self.cursor] -= 1;
                self.cursor = (self.cursor + 1) % FILE_CACHE_SIZE;
            }
        }
    }
}

#[test]
fn dummy_is_default() {
    assert_eq!(FileHash::default(), DUMMY_FILE_HASH);
}
