use crate::{DUMMY_FILE_HASH, last_modified, read_bytes, FileHash};
use crate::error::FileError;
use std::collections::HashMap;

// `FileCache` is not synchronized at all!
// It's caller's responsibility to use proper lock mechanisms

const FILE_CACHE_SIZE: usize = 128;
const SIZE_LIMIT: usize = 256 * 1024 * 1024;

type Path = String;

pub(crate) struct FileCache {
    data: Vec<(FileHash, Vec<u8>)>,

    // it's incremented everytime a file is read
    // when a file has to be removed, a file with the smallest count is removed
    count: Vec<u8>,

    // clock algorithm
    // points the most recently removed file's index
    cursor: usize,

    // sum of data's len
    total_size: usize,

    modified_times: HashMap<Path, u64>,
}

impl FileCache {
    pub fn new() -> Self {
        FileCache {
            data: vec![(DUMMY_FILE_HASH, vec![]); FILE_CACHE_SIZE],
            count: vec![0; FILE_CACHE_SIZE],
            cursor: 0,
            total_size: 0,
            modified_times: HashMap::new(),
        }
    }

    // TODO: lifetime of `self` and `[u8]` are different,
    // but the compiler doesn't know that -> this causes VERY VERY SERIOUS problems VERY VERY OFTEN
    pub fn get(&mut self, hash: FileHash) -> Option<&[u8]> {
        debug_assert_eq!(
            self.data.iter().map(
                |(_, file)| file.len()
            ).sum::<usize>(),
            self.total_size,
        );

        for i in 0..FILE_CACHE_SIZE {
            if self.data[i].0 == hash {
                self.count[i] = (self.count[i] + 1).min(128);
                return self.data.get(i).map(|(_, data)| data as &[u8]);
            }
        }

        None
    }

    pub fn insert(&mut self, hash: FileHash, path: &str) -> Result<(), FileError> {
        let min_count = *self.count.iter().min().unwrap();

        if min_count > 0 {
            for c in self.count.iter_mut() {
                *c -= min_count;
            }
        }

        // clock algorithm
        loop {
            if self.count[self.cursor] == 0 {
                match read_bytes(path) {
                    Ok(f) => {
                        // might have changed while waiting for the lock
                        if self.count[self.cursor] != 0 {
                            continue;
                        }

                        match last_modified(path) {
                            Ok(m) => {
                                match self.modified_times.get(path) {
                                    Some(m_) if *m_ != m => {
                                        return Err(FileError::modified_while_compilation(path));
                                    },
                                    Some(_) => { /* nop */ },
                                    None => {
                                        self.modified_times.insert(path.to_string(), m);
                                    },
                                }
                            },
                            Err(_) => {
                                return Err(FileError::metadata_not_supported(path));
                            },
                        }

                        self.total_size -= self.data[self.cursor].1.len();
                        self.total_size += f.len();

                        self.data[self.cursor] = (hash, f);
                        self.count[self.cursor] = 1;
                        self.cursor = (self.cursor + 1) % FILE_CACHE_SIZE;

                        if self.total_size > SIZE_LIMIT {
                            self.total_size -= self.data[self.cursor].1.len();
                            self.data[self.cursor] = (DUMMY_FILE_HASH, vec![]);
                            self.count[self.cursor] = 0;
                        }

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
