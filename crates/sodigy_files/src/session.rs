use crate::{DUMMY_FILE_HASH, IS_FILE_SESSION_INIT, LOCK};
use crate::cache::FileCache;
use crate::error::FileError;
use sodigy_test::{sodigy_log, LOG_NORMAL};
use std::collections::{hash_map, HashMap, HashSet};
use std::hash::Hasher;

pub type FileHash = u64;
pub type Path = String;

pub struct FileSession {
    tmp_files: HashMap<FileHash, Vec<u8>>,  // raw inputs
    files: HashMap<FileHash, Path>,
    tmp_files_rev: HashMap<Vec<u8>, FileHash>,
    files_rev: HashMap<Path, FileHash>,
    file_cache: FileCache,

    // it detects hash collisions
    hashes: HashSet<FileHash>,
    name_aliases: HashMap<FileHash, String>,
}

impl FileSession {
    /// Don't use this function! You should always use `global_file_session()`
    pub(crate) fn new() -> Self {
        sodigy_log!(LOG_NORMAL, String::from("FileSession::new: enter"));

        // prevent hasher from initing DUMMY_FILE_HASH accidentally
        let hashes = [DUMMY_FILE_HASH].into_iter().collect();

        unsafe {
            assert!(!IS_FILE_SESSION_INIT);
        }

        FileSession {
            tmp_files: HashMap::new(),
            files: HashMap::new(),
            tmp_files_rev: HashMap::new(),
            files_rev: HashMap::new(),
            hashes,
            file_cache: FileCache::new(),
            name_aliases: HashMap::new(),
        }
    }

    pub fn set_name_alias(&mut self, file: FileHash, name: String) {
        self.name_aliases.insert(file, name);
    }

    /// It returns the filename of the given `FileHash`.
    pub fn render_file_hash(&self, file: FileHash) -> String {
        match self.name_aliases.get(&file) {
            Some(n) => n.to_string(),
            None => match self.files.get(&file) {
                Some(p) => p.to_string(),
                _ => match self.tmp_files.get(&file) {
                    Some(_) => format!("tmp_{:x}", file & 0xfffffff),
                    _ => "FILE_NOT_FOUND".to_string(),
                }
            },
        }
    }

    /// It returns Err when there's a hash collision
    fn hash(&mut self, s: &[u8], is_tmp_file: bool) -> Result<FileHash, FileError> {
        let mut hasher = hash_map::DefaultHasher::new();
        hasher.write(s);
        let mut hash = hasher.finish();

        if is_tmp_file {
            hash |= 1 << 63;
        }

        else {
            hash &= !(1 << 63);
        }

        if self.hashes.contains(&hash) {
            return Err(FileError::hash_collision(
                &String::from_utf8_lossy(s).to_string()
            ));
        }

        self.hashes.insert(hash);

        Ok(hash)
    }

    pub fn get_file_name_from_hash(&self, hash: FileHash) -> Option<Path> {
        if let Some(name) = self.name_aliases.get(&hash) {
            Some(name.to_string())
        }

        else if let Some(path) = self.files.get(&hash) {
            Some(path.to_string())
        }

        else if self.tmp_files.contains_key(&hash) {
            Some(format!("tmp_{:x}", hash & 0xfffffff))
        }

        else {
            println!("{hash}, {:?}, {:?}", self.files, self.tmp_files);
            None
        }
    }

    /// It returns Err when there's a hash collision.
    pub fn register_tmp_file(&mut self, content: &[u8]) -> Result<FileHash, FileError> {
        sodigy_log!(LOG_NORMAL, format!("FileSession::register_tmp_file: {}", content.len()));

        let lock = unsafe { LOCK.lock().unwrap() };

        if let Some(f) = self.tmp_files_rev.get(content) {
            return Ok(*f);
        }

        let hash = self.hash(content, true)?;

        self.tmp_files.insert(
            hash,
            content.to_vec(),
        );

        self.tmp_files_rev.insert(
            content.to_vec(),
            hash,
        );

        drop(lock);

        Ok(hash)
    }

    pub fn try_register_hash_and_file(&mut self, hash: FileHash, path: &Path) -> Result<(), FileError> {
        let hashed = self.register_file(path)?;

        if hash != hashed {
            Err(FileError::hash_changed(path))
        }

        else {
            Ok(())
        }
    }

    /// It returns Err when there's a hash collision.
    pub fn register_file(&mut self, path: &Path) -> Result<FileHash, FileError> {
        sodigy_log!(LOG_NORMAL, format!("FileSession::register_file: {path}"));
        let lock = unsafe { LOCK.lock().unwrap() };

        if let Some(f) = self.files_rev.get(path) {
            return Ok(*f);
        }

        let hash = self.hash(path.as_bytes(), false)?;

        self.files.insert(
            hash,
            path.to_string(),
        );

        self.files_rev.insert(
            path.to_string(),
            hash,
        );

        drop(lock);

        Ok(hash)
    }

    pub fn get_file_content(&mut self, hash: FileHash) -> Result<&[u8], FileError> {
        sodigy_log!(LOG_NORMAL, format!("FileSession::get_file_content: {hash}"));

        match self.get_fs_file_content(hash) {
            // it's just `Ok(v)`
            // the compiler thinks `v` and `self.get_fs_file_content` violates the borrow rules,
            // but they don't! It's a limitation of the current borrow checker
            // the Rust team says the next version of the borrow checker will fix this
            Ok(v) => unsafe { Ok(&*(v as *const [u8])) },
            Err(e) => match self.get_tmp_file(hash) {
                Some(b) => Ok(b),
                None => Err(e),
            }
        }
    }

    fn get_tmp_file(&self, hash: FileHash) -> Option<&[u8]> {
        self.tmp_files.get(&hash).map(|f| f as &[u8])
    }

    fn get_fs_file_content(&mut self, hash: FileHash) -> Result<&[u8], FileError> {
        let lock = unsafe { LOCK.lock().unwrap() };

        match self.file_cache.get(hash) {
            // it's just `Ok(v)`
            // the compiler thinks `v` and `self.get_fs_file_content` violates the borrow rules,
            // but they don't! It's a limitation of the current borrow checker
            // the Rust team says the next version of the borrow checker will fix this
            Some(v) => unsafe {
                drop(lock);

                Ok(&*(v as *const [u8]))
            },

            None => {
                let path = match self.files.get(&hash) {
                    Some(p) => p,
                    None => {
                        return Err(FileError::invalid_file_hash(hash));
                    },
                };

                self.file_cache.insert(hash, path)?;
                drop(lock);

                self.get_fs_file_content(hash)
            },
        }
    }
}
