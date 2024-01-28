use crate::FileHash;
use std::collections::hash_map;
use std::ffi::OsString;
use std::hash::Hasher;
use std::io;

#[derive(Clone, Debug)]
pub struct FileError {
    pub kind: FileErrorKind,
    pub given_path: Option<String>,
    pub context: FileErrorContext,
}

impl FileError {
    pub fn from_std(e: io::Error, given_path: &str) -> Self {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => FileErrorKind::FileNotFound,
            io::ErrorKind::PermissionDenied => FileErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => FileErrorKind::AlreadyExists,
            _ => FileErrorKind::Unknown(format!("{e:?}")),
        };

        FileError {
            kind,
            given_path: Some(given_path.to_string()),
            context: FileErrorContext::None,
        }
    }

    pub fn set_context(&mut self, context: FileErrorContext) -> &mut Self {
        if self.context == FileErrorContext::None {
            self.context = context;
        }

        self
    }

    pub fn invalid_file_hash(hash: FileHash) -> Self {
        FileError {
            kind: FileErrorKind::InvalidFileHash(hash),
            given_path: None,
            context: FileErrorContext::None,
        }
    }

    pub fn modified_while_compilation(given_path: &str) -> Self {
        FileError {
            kind: FileErrorKind::ModifiedWhileCompilation,
            given_path: Some(given_path.to_string()),
            context: FileErrorContext::None,
        }
    }

    pub fn metadata_not_supported(given_path: &str) -> Self {
        FileError {
            kind: FileErrorKind::MetadataNotSupported,
            given_path: Some(given_path.to_string()),
            context: FileErrorContext::None,
        }
    }

    pub fn hash_collision(given_path: &str) -> Self {
        FileError {
            kind: FileErrorKind::HashCollision,
            given_path: Some(given_path.to_string()),
            context: FileErrorContext::None,
        }
    }

    pub fn hash_changed(path: &str) -> Self {
        FileError {
            kind: FileErrorKind::HashChanged,
            given_path: Some(path.to_string()),
            context: FileErrorContext::None,
        }
    }

    pub fn cannot_create_file(there_exists_a_dir: bool, path: &str) -> Self {
        FileError {
            kind: FileErrorKind::CannotCreateFile { there_exists_a_dir },
            given_path: Some(path.to_string()),
            context: FileErrorContext::None,
        }
    }

    pub fn unknown(msg: String, path: Option<String>) -> Self {
        FileError {
            kind: FileErrorKind::Unknown(msg),
            given_path: path,
            context: FileErrorContext::None,
        }
    }

    pub(crate) fn os_str_err(os_str: OsString) -> Self {
        FileError {
            kind: FileErrorKind::OsStrErr(os_str),
            given_path: None,
            context: FileErrorContext::None,
        }
    }

    pub fn render_error(&self) -> String {
        let path = self.given_path.as_ref().map(|p| p.to_string()).unwrap_or(String::new());

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
            FileErrorKind::Unknown(msg) => format!(
                "unknown file error: `{msg}`"
            ),
            FileErrorKind::OsStrErr(os_str) => format!(
                "error converting os_str: `{os_str:?}`"
            ),
            FileErrorKind::InvalidFileHash(hash) => format!(
                "invalid file hash: {hash}"
            ),
            FileErrorKind::MetadataNotSupported => format!(
                "unable to read file metadata: `{path}`"
            ),
            FileErrorKind::ModifiedWhileCompilation => format!(
                "source file modified while compilation: `{path}`"
            ),
            FileErrorKind::HashCollision => format!(
                "hash collision: `{path}`"
            ),
            FileErrorKind::HashChanged => format!(
                "broken hash value of `{path}`"
            ),
            FileErrorKind::CannotCreateFile { there_exists_a_dir } => {
                let (has_to_create, there_exists) = if *there_exists_a_dir {
                    ("file", "directory")
                } else {
                    ("directory", "file")
                };

                format!(
                    "cannot create {has_to_create}: `{path}`\nIt has to create a {has_to_create} named `{path}`, but there exists a {there_exists} with the same name.",
                )
            },
        }
    }

    pub fn hash_u64(&self) -> u64 {
        let mut hasher = hash_map::DefaultHasher::new();
        hasher.write(&self.kind.hash_u64().to_be_bytes());

        if let Some(p) = &self.given_path {
            hasher.write(p.as_bytes());
        }

        hasher.finish()
    }

    pub fn is_file_not_found_error(&self) -> bool {
        matches!(self.kind, FileErrorKind::FileNotFound)
    }
}

#[derive(Clone, Debug)]
pub enum FileErrorKind {
    FileNotFound,
    PermissionDenied,
    AlreadyExists,
    OsStrErr(OsString),
    Unknown(String),

    // Sodigy specific errors from here
    InvalidFileHash(FileHash),
    MetadataNotSupported,
    ModifiedWhileCompilation,
    HashCollision,

    // only returned by `FileSession::try_register_hash_and_file`
    //
    // Some objects deal with hash values of paths.
    // if the hash values of the same path changes (mostly when the compiler is updated),
    // this error is thrown
    HashChanged,

    // Its name is misleading... but I can't think of any better one
    // it's raised when
    // 1. it has to make a file named X, but there exists a dir named X
    // 2. vice versa
    CannotCreateFile { there_exists_a_dir: bool },
}

impl FileErrorKind {
    pub fn hash_u64(&self) -> u64 {
        match self {
            FileErrorKind::FileNotFound => 0,
            FileErrorKind::PermissionDenied => 1,
            FileErrorKind::AlreadyExists => 2,
            FileErrorKind::OsStrErr(s) => {
                let mut hasher = hash_map::DefaultHasher::new();
                hasher.write(s.as_encoded_bytes());

                hasher.finish()
            },
            FileErrorKind::Unknown(s) => {
                let mut hasher = hash_map::DefaultHasher::new();
                hasher.write(s.as_bytes());

                hasher.finish()
            },
            FileErrorKind::InvalidFileHash(h) => {
                let mut hasher = hash_map::DefaultHasher::new();
                hasher.write(&h.to_be_bytes());

                hasher.finish()
            },
            FileErrorKind::MetadataNotSupported => 3,
            FileErrorKind::ModifiedWhileCompilation => 4,
            FileErrorKind::HashCollision => 5,
            FileErrorKind::HashChanged => 6,
            FileErrorKind::CannotCreateFile { there_exists_a_dir } => {
                ((*there_exists_a_dir as u64) << 4) | 7
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileErrorContext {
    None,
    SavingIr,
    CleaningIr,
    DumpingTokensToFile,
    DumpingHirToFile,
}
