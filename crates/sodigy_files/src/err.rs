use std::ffi::OsString;
use std::io;

// TODO: impl SodigyError for this type
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
}

impl FileError {
    pub fn init(e: io::Error, given_path: &str) -> Self {
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

    pub(crate) fn os_str_err(os_str: OsString) -> Self {
        FileError {
            kind: FileErrorKind::OsStrErr(os_str),
            given_path: None,
        }
    }

    pub fn render_err(&self) -> String {
        let path = match self.kind {
            FileErrorKind::FileNotFound
            | FileErrorKind::PermissionDenied
            | FileErrorKind::AlreadyExists => {
                self.given_path.as_ref().expect(
                    "Internal Compiler Error AD3764202D8"
                ).to_string()
            },
            FileErrorKind::OsStrErr(_) => String::new(),
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
        }
    }
}
