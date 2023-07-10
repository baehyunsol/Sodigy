// TODO: It must be at another crate, not in the parser!

use std::fs::{read, File};
use std::io::{self, Read};

#[derive(Debug)]
pub struct FileError {
    kind: FileErrorKind,
    given_path: Option<String>,
}

#[derive(Debug)]
pub enum FileErrorKind {
    FileNotFound,
    PermissionDenied,
}

impl FileError {

    pub fn init(e: io::Error, given_path: &str) -> Self {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => FileErrorKind::FileNotFound,
            io::ErrorKind::PermissionDenied => FileErrorKind::PermissionDenied,
            _ => todo!(),
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