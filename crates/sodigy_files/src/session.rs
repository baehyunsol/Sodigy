use crate::LOCK;
use crate::err::FileError;
use crate::funcs::*;
use std::collections::{hash_map, HashMap, HashSet};
use std::hash::Hasher;

pub type FileHash = u64;
pub type Path = String;

pub struct Session {
    tmp_files: HashMap<FileHash, Vec<u8>>,  // used for tests
    files: HashMap<FileHash, Path>,
    hashes: HashSet<FileHash>,
}

impl Session {
    /// It shall not be called directly.
    pub(crate) fn new() -> Self {
        Session {
            tmp_files: HashMap::new(),
            files: HashMap::new(),
            hashes: HashSet::new(),
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

    pub fn get_file_content(&self, hash: FileHash) -> Result<Vec<u8>, FileError> {
        match self.files.get(&hash) {
            Some(path) => read_bytes(path),
            None => Ok(self.get_tmp_file(hash).unwrap().to_vec()),
        }
    }
}
