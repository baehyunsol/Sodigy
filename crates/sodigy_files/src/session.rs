use crate::{DUMMY_FILE_HASH, LOCK};
use crate::cache::FileCache;
use crate::err::FileError;
use std::collections::{hash_map, HashMap, HashSet};
use std::hash::Hasher;

pub type FileHash = u64;
pub type Path = String;

pub struct Session {
    tmp_files: HashMap<FileHash, Vec<u8>>,  // used for tests
    files: HashMap<FileHash, Path>,
    file_cache: FileCache,
    hashes: HashSet<FileHash>,
}

impl Session {
    /// It shall not be called directly.
    pub(crate) fn new() -> Self {
        // prevent hasher from initing DUMMY_FILE_HASH accidentally
        let hashes = [DUMMY_FILE_HASH].into_iter().collect();

        Session {
            tmp_files: HashMap::new(),
            files: HashMap::new(),
            hashes,
            file_cache: FileCache::new(),
        }
    }

    pub fn render_file_hash(&self, file: FileHash) -> String {
        match self.files.get(&file) {
            Some(p) => format!("{p}"),
            _ => match self.tmp_files.get(&file) {
                Some(_) => format!("tmp_{:x}", file & 0xfffffff),
                _ => "FILE_NOT_FOUND".to_string(),
            }
        }
    }

    fn hash(&mut self, s: &[u8]) -> FileHash {
        let mut hasher = hash_map::DefaultHasher::new();
        hasher.write(s);
        let mut hash = hasher.finish();

        let lock = unsafe { LOCK.lock() };

        while self.hashes.contains(&hash) {
            hash += 1;
        }

        self.hashes.insert(hash);

        drop(lock);

        return hash;
    }

    pub fn register_tmp_file(&mut self, content: Vec<u8>) -> FileHash {
        let hash = self.hash(&content);

        self.tmp_files.insert(
            hash,
            content,
        );

        hash
    }

    pub fn register_file(&mut self, path: &Path) -> FileHash {
        let hash = self.hash(path.as_bytes());

        self.files.insert(
            hash,
            path.to_string(),
        );

        hash
    }

    pub fn get_tmp_file(&self, hash: FileHash) -> Option<&[u8]> {
        self.tmp_files.get(&hash).map(|f| f as &[u8])
    }

    pub fn get_file_content(&mut self, hash: FileHash) -> Result<&[u8], FileError> {
        match self.file_cache.get(hash) {
            // it's just `Ok(v)`
            // the compiler thinks `v` and `self.get_file_content` violates the borrow rules,
            // but they don't! It's a limitation of the current borrow checker
            // the Rust team says the next version of the borrow checker will fix this
            Some(v) => unsafe { Ok(&*(v as *const [u8])) },

            None => {
                self.file_cache.insert(hash, self.files.get(&hash).unwrap())?;

                self.get_file_content(hash)
            },
        }
    }
}
