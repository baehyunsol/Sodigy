use std::fs::{read, File, OpenOptions};
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct FileError {
    kind: FileErrorKind,
    given_path: Option<String>,
}

#[derive(Debug)]
pub enum FileErrorKind {
    FileNotFound,
    PermissionDenied,
    AlreadyExists,
}

/// ```nohighlight
///       File Already Exists    File Does not Exist
///
///     AA       Append                  Dies
///    AoC       Append                 Create
///    CoT      Truncate                Create
///     AC        Dies                  Create
/// ```
pub enum WriteMode {
    AlwaysAppend,
    AppendOrCreate,
    CreateOrTruncate,
    AlwaysCreate,
}

impl From<WriteMode> for OpenOptions {
    fn from(m: WriteMode) -> OpenOptions {
        let mut result = OpenOptions::new();

        match m {
            WriteMode::AlwaysAppend => { result.append(true); },
            WriteMode::AppendOrCreate => { result.append(true).create(true); }
            WriteMode::CreateOrTruncate => { result.write(true).truncate(true).create(true); }
            WriteMode::AlwaysCreate => { result.write(true).create_new(true); }
        }

        result
    }
}

impl FileError {

    pub fn init(e: io::Error, given_path: &str) -> Self {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => FileErrorKind::FileNotFound,
            io::ErrorKind::PermissionDenied => FileErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => FileErrorKind::AlreadyExists,
            _ => panic!("{e:?}"),
        };

        FileError {
            kind, given_path: Some(given_path.to_string())
        }
    }

}

pub fn read_bytes(path: &str) -> Result<Vec<u8>, FileError> {

    match read(path) {
        Ok(data) => Ok(data),
        Err(e) => Err(FileError::init(e, path)),
    }

}

pub fn read_string(path: &str) -> Result<String, FileError> {
    let mut s = String::new();

    match File::open(path) {
        Err(e) => Err(FileError::init(e, path)),
        Ok(mut f) => match f.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(e) => Err(FileError::init(e, path)),
        }
    }

}

pub fn write_bytes(path: &str, bytes: &[u8], write_mode: WriteMode) -> Result<(), FileError> {
    let option: OpenOptions = write_mode.into();

    match option.open(path) {
        Ok(mut f) => match f.write_all(bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(FileError::init(e, path)),
        },
        Err(e) => Err(FileError::init(e, path)),
    }

}

pub fn write_string(path: &str, s: &str, write_mode: WriteMode) -> Result<(), FileError> {
    write_bytes(path, s.as_bytes(), write_mode)
}