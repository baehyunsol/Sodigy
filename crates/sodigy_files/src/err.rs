use crate::FileHash;
use std::ffi::OsString;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct FileError {
    kind: FileErrorKind,
    given_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileErrorKind {
    FileNotFound,
    PermissionDenied,
    AlreadyExists,
    OsStrErr(OsString),

    // Sodigy specific errors
    InvalidFileHash(FileHash),
    MetadataNotSupported,
    ModifiedWhileCompilation,
    HashCollision,
}

impl FileError {
    pub fn from_std(e: io::Error, given_path: &str) -> Self {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => FileErrorKind::FileNotFound,
            io::ErrorKind::PermissionDenied => FileErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => FileErrorKind::AlreadyExists,
            _ => panic!("e: {e:?}, path: {given_path}"),
        };

        FileError {
            kind, given_path: Some(given_path.to_string())
        }
    }

    pub fn invalid_file_hash(hash: FileHash) -> Self {
        FileError {
            kind: FileErrorKind::InvalidFileHash(hash),
            given_path: None,
        }
    }

    pub fn modified_while_compilation(given_path: &str) -> Self {
        FileError {
            kind: FileErrorKind::ModifiedWhileCompilation,
            given_path: Some(given_path.to_string()),
        }
    }

    pub fn metadata_not_supported(given_path: &str) -> Self {
        FileError {
            kind: FileErrorKind::MetadataNotSupported,
            given_path: Some(given_path.to_string()),
        }
    }

    pub fn hash_collision(given_path: &str) -> Self {
        FileError {
            kind: FileErrorKind::HashCollision,
            given_path: Some(given_path.to_string()),
        }
    }

    pub(crate) fn os_str_err(os_str: OsString) -> Self {
        FileError {
            kind: FileErrorKind::OsStrErr(os_str),
            given_path: None,
        }
    }

    pub fn render_error(&self) -> String {
        let path = match self.kind {
            FileErrorKind::FileNotFound
            | FileErrorKind::PermissionDenied
            | FileErrorKind::AlreadyExists
            | FileErrorKind::MetadataNotSupported
            | FileErrorKind::HashCollision
            | FileErrorKind::ModifiedWhileCompilation => {
                self.given_path.as_ref().unwrap().to_string()
            },
            FileErrorKind::OsStrErr(_)
            | FileErrorKind::InvalidFileHash(_) => String::new(),
        };

        match &self.kind {
            FileErrorKind::FileNotFound => format!(
                "file not found: `{path}`"
            ),
            FileErrorKind::PermissionDenied => format!(
                "permission denied: `{path}`",
            ),
            FileErrorKind::AlreadyExists => format!(
                "file already exists: `{path}`"
            ),
            FileErrorKind::OsStrErr(os_str) => format!(
                "error converting os_str: `{os_str:?}`"
            ),
            FileErrorKind::InvalidFileHash(hash) => format!(
                "invalid file hash: {hash}"
            ),
            FileErrorKind::MetadataNotSupported => String::from(
                "unable to read file metadata"
            ),
            FileErrorKind::ModifiedWhileCompilation => String::from(
                "source file modified while compilation"
            ),
            FileErrorKind::HashCollision => format!(
                "hash collision: `{path}`"
            ),
        }
    }
}
