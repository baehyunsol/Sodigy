use crate::std_file::{STD_FILES, STD_FILE_INDEXES};
use sodigy_fs_api::{FileError, join3, read_bytes};
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

// The four most significant bits are for the kind (`FileKind`).
//   0: source file
//   1: std
//   2: object file
// The other 28 bit is the id of the file.
//
// Its `Ord` is for deterministic output of the error messages (it sorts the errors by file).
// It doesn't sort the files by name.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileKind {
    Source,
    Std,
    ObjectFile,
}

// TODO: rename this to `File`
pub struct FileInfo {
    id: FileId,
    module_path: String,
    content: InternedString,

    // This is shown when rendering spans.
    // You shouldn't use this path to read the file, use the `.content` field!
    file_path: Option<String>,
}

impl FileId {
    pub fn std(id: u32) -> Self {
        FileId(0x1000_0000 | (id & 0x0fff_ffff))
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn from_u32(id: u32) -> Self {
        FileId(id)
    }

    pub fn kind(&self) -> FileKind {
        match self.0 >> 28 {
            0 => FileKind::Source,
            1 => FileKind::Std,
            2 => FileKind::ObjectFile,
            _ => unreachable!(),
        }
    }

    pub fn register(
        // `read_bytes(file_path)` should work
        file_path: &str,

        // Each module has a unique module_path within a project.
        // It's like file_path, but represents a module hierarchy.
        // For example, a module with `foo/bar` can be in `src/foo/bar.rs` or `src/foo/bar/mod.rs`.
        module_path: &str,

        kind: FileKind,
        intermediate_dir: &str,
    ) -> Result<FileId, FileError> {
        todo!()
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

impl fmt::Debug for FileId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.kind() {
            FileKind::Source => write!(formatter, "FileId({})", self.0),
            FileKind::Std => {
                let s = STD_FILES[*STD_FILE_INDEXES.get(&self.0).unwrap()].file_path_str.to_string();
                write!(formatter, "FileId({s:?})")
            },
            FileKind::ObjectFile => write!(formatter, "FileId(ObjectFile)"),
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

