use crate::std_file::{STD_FILES, STD_FILE_INDEXES};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    exists,
    join,
    join3,
    read_bytes,
    write_bytes,
};
use sodigy_string::{InternedString, intern_string, unintern_string};
use sodigy_utils::hash;
use std::fmt;
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

// The four most significant bits are the type of the file
//   0: source file
//   1: std
//   2: object file
// The other 28 bit is the id of the file.
//
// Its `Ord` is for deterministic output of the error messages (it sorts the errors by file).
// It doesn't sort the files by name.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct File(u32);

impl File {
    pub fn std(id: u32) -> Self {
        File(0x1000_0000 | (id & 0x0fff_ffff))
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn from_id(id: u32) -> Self {
        File(id)
    }

    pub fn register(
        // `read_bytes(file_path)` should work
        file_path: &str,

        // Each module has a unique module_path within a project.
        // It's like file_path, but represents a module hierarchy.
        // For example, a module with `foo/bar` can be in `src/foo/bar.rs` or `src/foo/bar/mod.rs`.
        module_path: &str,

        intermediate_dir: &str,
        is_object_file: bool,
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
        let mut file_id = if file_map.is_empty() {
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
        file_id &= 0x0fff_ffff;

        if is_object_file {
            file_id |= 0x2000_0000;
        }

        let content = read_bytes(file_path)?;
        let content_hash = intern_string(&content, intermediate_dir)?;
        push_file_map(&mut file_map, file_id, content_hash.0, module_path, file_path);
        file_map_fd.write_all(&file_map).map_err(
            |e| FileError::from_std(e, &file_map_path)
        )?;
        file_map_fd.unlock().map_err(|e| FileError::from_std(e, &file_map_path))?;

        Ok(File(file_id))
    }

    pub fn from_module_path(module_path: &str, intermediate_dir: &str) -> Result<File, FileError> {
        let module_path_hash = (hash(module_path.as_bytes()) & 0xff) as u32;
        let file_map_path = join3(
            intermediate_dir,
            "file_map",
            &format!("{module_path_hash:02x}"),
        )?;
        let file_map = read_bytes(&file_map_path).unwrap_or(vec![]);

        match search_file_map_by_module_path(&file_map, module_path, &file_map_path)? {
            Some((file_id, _)) => Ok(File(file_id)),
            None => {
                for std_file in STD_FILES.iter() {
                    if std_file.module_path_str == module_path {
                        return Ok(File(std_file.file_hash()));
                    }
                }

                unreachable!()
            },
        }
    }

    // It returns (module_path, file_path).
    // This is very very expensive.
    pub fn get_path(&self, intermediate_dir: &str) -> Result<(String, String), FileError> {
        let r#type = self.0 >> 28;

        match r#type {
            0 | 2 => {
                let file_map_path = join3(
                    intermediate_dir,
                    "file_map",
                    &format!("{:02x}", self.0 & 0xff),
                )?;
                let file_map = read_bytes(&file_map_path)?;

                match search_file_map_by_id(&file_map, self.0, &file_map_path)? {
                    Some((module_path, file_path, _)) => Ok((module_path.to_string(), file_path.to_string())),
                    None => panic!(),  // error? panic? unreachable?
                }
            },
            1 => Ok((
                STD_FILES[*STD_FILE_INDEXES.get(&self.0).unwrap()].module_path_str.to_string(),
                STD_FILES[*STD_FILE_INDEXES.get(&self.0).unwrap()].file_path_str.to_string(),
            )),
            _ => unreachable!(),
        }
    }

    pub fn get_content_hash(&self, intermediate_dir: &str) -> Result<u128, FileError> {
        let r#type = self.0 >> 28;

        match r#type {
            0 | 2 => {
                let file_map_path = join3(
                    intermediate_dir,
                    "file_map",
                    &format!("{:02x}", self.0 & 0xff),
                )?;
                let file_map = read_bytes(&file_map_path)?;

                match search_file_map_by_id(&file_map, self.0, &file_map_path)? {
                    Some((_, _, content_hash)) => Ok(content_hash),

                    // error? panic? unreachable?
                    None => todo!(),
                }
            },
            1 => Ok(STD_FILES[*STD_FILE_INDEXES.get(&self.0).unwrap()].file_hash() as u128),
            _ => unreachable!(),
        }
    }

    // This is very very expensive.
    pub fn read_bytes(&self, intermediate_dir: &str) -> Result<Vec<u8>, FileError> {
        let r#type = self.0 >> 28;

        match r#type {
            0 | 2 => {
                let content_hash = self.get_content_hash(intermediate_dir)?;
                Ok(unintern_string(InternedString(content_hash), intermediate_dir)?.unwrap())
            },
            1 => Ok(STD_FILES[*STD_FILE_INDEXES.get(&self.0).unwrap()].contents.to_vec()),
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for File {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let r#type = self.0 >> 28;

        match r#type {
            0 => write!(formatter, "File({})", self.0),
            1 => {
                let s = STD_FILES[*STD_FILE_INDEXES.get(&self.0).unwrap()].file_path_str.to_string();
                write!(formatter, "File({s:?})")
            },
            2 => write!(formatter, "File(ObjectFile)"),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum FileOrStd {
    File(String),
    Std(u32),
}

impl fmt::Debug for FileOrStd {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            FileOrStd::File(s) => write!(formatter, "File({s:?})"),
            FileOrStd::Std(n) => {
                let s = STD_FILES[*STD_FILE_INDEXES.get(n).unwrap()].file_path_str.to_string();
                write!(formatter, "Std({s:?})")
            },
        }
    }
}

