use crate::std_file::STD_FILES;
use sodigy_fs_api::{FileError, join3, read_bytes};
use sodigy_string::{InternedString, hash, intern_string, unintern_string};
use std::fs::File as StdFile;
use std::io::{Read, Write};

mod endec;
mod error;
mod file_map;
mod module_path;
mod std_file;

pub use error::GetFilePathError;
pub use module_path::ModulePath;
pub use std_file::std_root;

use file_map::{
    length_file_map,
    push_file_map,
    search_file_map_by_id,
    search_file_map_by_module_path,
};

// The most significant bit tells whether it's std or not (1 if std).
// The other 31 bit is the id of the file.
//
// Its `Ord` is for deterministic output of the error messages (it sorts the errors by file).
// It doesn't sort the files by name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct File(pub u32);

impl File {
    pub fn std(id: u32) -> Self {
        File(0x8000_0000 | id)
    }

    pub fn register(
        // `read_bytes(file_path)` should work
        file_path: &str,

        // Each module has a unique module_path within a project.
        // It's like file_path, but represents a module hierarchy.
        // For example, a module with `foo/bar` can be in `src/foo/bar.rs` or `src/foo/bar/mod.rs`.
        module_path: &str,

        intermediate_dir: &str,
    ) -> Result<File, FileError> {
        let module_path_hash = (hash(module_path.as_bytes()) & 0xff) as u32;
        let file_map_path = join3(
            intermediate_dir,
            "file_map",
            &format!("{module_path_hash:02x}"),
        )?;
        let mut file_map_fd = StdFile::options()
            .read(true)
            .create(true)
            .append(true)
            .open(&file_map_path)
            .map_err(|e| FileError::from_std(e, &file_map_path))?;

        file_map_fd.lock().map_err(
            |e| FileError::from_std(e, &file_map_path)
        )?;

        let mut file_map = vec![];
        file_map_fd.read_to_end(&mut file_map).map_err(
            |e| FileError::from_std(e, &file_map_path)
        )?;

        // file_map is a list of `file_id: u32`, `content_hash: u128`, `module_path: String`
        let file_id = if file_map.is_empty() {
            module_path_hash
        } else {
            match search_file_map_by_module_path(&file_map, module_path, &file_map_path)? {
                // If it's already registered, it returns the previous one without updating its content_hash.
                // That means you cannot update a file while a compilation is going on.
                Some((file_id, _)) => {
                    return Ok(File(file_id));
                },
                None => ((length_file_map(&file_map, &file_map_path)? as u32) << 8) | module_path_hash,
            }
        };

        let content = read_bytes(file_path)?;
        let content_hash = intern_string(&content, intermediate_dir)?;
        push_file_map(&mut file_map, file_id, content_hash.0, module_path, file_path);
        file_map_fd.write_all(&file_map).map_err(
            |e| FileError::from_std(e, &file_map_path)
        )?;
        file_map_fd.unlock().map_err(|e| FileError::from_std(e, &file_map_path))?;

        Ok(File(file_id))
    }

    pub fn from_module_path(module_path: &str, intermediate_dir: &str) -> Result<Option<File>, FileError> {
        let module_path_hash = (hash(module_path.as_bytes()) & 0xff) as u32;
        let file_map_path = join3(
            intermediate_dir,
            "file_map",
            &format!("{module_path_hash:02x}"),
        )?;
        let file_map = read_bytes(&file_map_path).unwrap_or(vec![]);

        match search_file_map_by_module_path(&file_map, module_path, &file_map_path)? {
            Some((file_id, _)) => Ok(Some(File(file_id))),
            None => {
                for (i, (p, _, _, _)) in STD_FILES.iter().enumerate() {
                    if *p == module_path {
                        return Ok(Some(File(0x8000_0000 | i as u32)));
                    }
                }

                Ok(None)
            },
        }
    }

    // It returns (module_path, file_path).
    // This is very very expensive.
    pub fn get_path(&self, intermediate_dir: &str) -> Result<Option<(String, String)>, FileError> {
        let is_std = self.0 >= 0x8000_0000;
        let id = self.0 & 0x7fff_ffff;

        if is_std {
            Ok(Some((
                STD_FILES[id as usize].0.to_string(),
                STD_FILES[id as usize].1.to_string(),
            )))
        } else {
            let file_map_path = join3(
                intermediate_dir,
                "file_map",
                &format!("{:02x}", id & 0xff),
            )?;
            let file_map = read_bytes(&file_map_path)?;

            match search_file_map_by_id(&file_map, id, &file_map_path)? {
                Some((module_path, file_path, _)) => Ok(Some((module_path.to_string(), file_path.to_string()))),
                None => Ok(None),
            }
        }
    }

    pub fn get_content_hash(&self, intermediate_dir: &str) -> Result<u128, FileError> {
        let is_std = self.0 >= 0x8000_0000;
        let id = self.0 & 0x7fff_ffff;

        if is_std {
            Ok(STD_FILES[id as usize].3)
        } else {
            let file_map_path = join3(
                intermediate_dir,
                "file_map",
                &format!("{:02x}", id & 0xff),
            )?;
            let file_map = read_bytes(&file_map_path)?;

            match search_file_map_by_id(&file_map, id, &file_map_path)? {
                Some((_, _, content_hash)) => Ok(content_hash),

                // error? panic? unreachable?
                None => todo!(),
            }
        }
    }

    // This is very very expensive.
    pub fn read_bytes(&self, intermediate_dir: &str) -> Result<Option<Vec<u8>>, FileError> {
        let is_std = self.0 >= 0x8000_0000;
        let id = self.0 & 0x7fff_ffff;

        if is_std {
            Ok(Some(STD_FILES[id as usize].2.to_vec()))
        } else {
            let content_hash = self.get_content_hash(intermediate_dir)?;
            Ok(unintern_string(InternedString(content_hash), intermediate_dir)?)
        }
    }
}

#[derive(Clone, Debug)]
pub enum FileOrStd {
    File(String),
    Std(u32),
}
