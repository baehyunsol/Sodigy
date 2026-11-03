use crate::std_file::{StdFile, STD_FILES, STD_FILE_BY_MODULE_PATH, STD_FILE_INDEXES};
use sodigy_fs_api::{FileError, join3, read_bytes};
use sodigy_string::{InternedString, intern_string, unintern_string};
use sodigy_utils::hash;
use std::fmt;
use std::io::{Read, Write};

mod endec;
mod error;
mod file_map;
mod module_path;
mod std_file;

pub use error::GetFilePathError;
pub use module_path::ModulePath;
pub use std_file::std_root;

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

impl FileKind {
    pub fn to_u32(&self) -> u32 {
        match self {
            FileKind::Source => 0,
            FileKind::Std => 1,
            FileKind::ObjectFile => 2,
        }
    }

    pub fn from_u32(n: u32) -> Option<FileKind> {
        match n {
            0 => Some(FileKind::Source),
            1 => Some(FileKind::Std),
            2 => Some(FileKind::ObjectFile),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct File {
    pub id: FileId,
    pub module_path: String,
    pub content: InternedString,

    // This is shown when rendering spans.
    // You shouldn't use this path to read the file, use the `.content` field!
    pub file_path: String,
}

impl FileId {
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn from_u32(id: u32) -> Self {
        FileId(id)
    }

    pub fn new(id: u32, kind: FileKind) -> Self {
        FileId((kind.to_u32() << 28) | (id & 0x0fff_ffff))
    }

    pub fn kind(&self) -> FileKind {
        FileKind::from_u32(self.0 >> 28).unwrap()
    }

    pub fn register(file_path: &str, module_path: &str, kind: FileKind, intermediate_dir: &str) -> Result<FileId, FileError> {
        let bytes = read_bytes(file_path)?;
        let content = intern_string(&bytes, intermediate_dir)?;
        let id = FileId::allocate_or_load_id(module_path, kind, intermediate_dir)?;
        let file = File { id, module_path: module_path.to_string(), content, file_path: file_path.to_string() };
        file.update(intermediate_dir)?;
        Ok(file.id)
    }

    pub fn register_std(id: u32, intermediate_dir: &str) -> Result<Option<FileId>, FileError> {
        let Some(std_file) = StdFile::from_hash(id) else { return Ok(None); };
        let id = FileId::new(id, FileKind::Std);
        let content = intern_string(std_file.contents, intermediate_dir)?;
        let file = File { id, module_path: std_file.module_path_str.to_string(), content, file_path: std_file.file_path_str.to_string() };
        file.update(intermediate_dir)?;
        Ok(Some(file.id))
    }

    pub fn from_module_path(module_path: &str, kind: FileKind, intermediate_dir: &str) -> Result<FileId, FileError> {
        match kind {
            FileKind::Std => Ok(FileId(STD_FILES[*STD_FILE_BY_MODULE_PATH.get(module_path).unwrap()].file_path.unwrap_std_id())),
            FileKind::Source | FileKind::ObjectFile => FileId::allocate_or_load_id(module_path, kind, intermediate_dir),
        }
    }

    pub fn get_info(&self, intermediate_dir: &str) -> Result<Option<File>, FileError> {
        File::search_by_id(*self, intermediate_dir)
    }

    pub fn read_bytes(&self, intermediate_dir: &str) -> Result<Option<Vec<u8>>, FileError> {
        let Some(file_info) = self.get_info(intermediate_dir)? else { return Ok(None); };
        unintern_string(file_info.content, intermediate_dir)
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

impl FileOrStd {
    pub fn unwrap_std_id(&self) -> u32 {
        match self {
            FileOrStd::File(_) => panic!(),
            FileOrStd::Std(id) => *id,
        }
    }
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

