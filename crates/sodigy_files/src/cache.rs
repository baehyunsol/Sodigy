use crate::{read_bytes, FileHash};
use crate::err::FileError;

#[cfg(test)]
use crate::DUMMY_FILE_HASH;

const FILE_CACHE_SIZE: usize = 32;

pub(crate) struct FileCache {
    data: [(FileHash, Vec<u8>); FILE_CACHE_SIZE],

    // it's incremented everytime a file is read
    // when a file has to be removed, a file with the smallest count is removed
    count: [usize; FILE_CACHE_SIZE],

    // points the most recently removed file's index
    cursor: usize,

    // sum of data's len
    // TODO: do something when `total_size` is too big
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

    pub fn get(&mut self, hash: FileHash) -> Option<&[u8]> {
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
                    Ok(f) => {
                        self.total_size -= self.data[self.cursor].1.len();
                        self.total_size += f.len();

                        self.data[self.cursor] = (hash, f);
                        self.count[self.cursor] = 1;
                        self.cursor += 1;
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
