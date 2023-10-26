// #![allow(dead_code)]
use crate::err::FileError;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

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

pub fn read_bytes(path: &str) -> Result<Vec<u8>, FileError> {

    match fs::read(path) {
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

// `a/b/c.d` -> `c`
pub fn file_name(path: &str) -> Result<String, FileError> {
    let path_buf = PathBuf::from_str(path).expect("Internal Compiler Error 26A88CF9FFC");  // it's infallible

    match path_buf.file_stem() {
        None => Ok(String::new()),
        Some(s) => match s.to_str() {
            Some(ext) => Ok(ext.to_string()),
            None => Err(FileError::os_str_err(s.to_os_string())),
        }
    }
}

// `a/b/c.d` -> `d`
pub fn extension(path: &str) -> Result<Option<String>, FileError> {
    let path_buf = PathBuf::from_str(path).expect("Internal Compiler Error CFF9FC88A62");  // it's infallible

    match path_buf.extension() {
        None => Ok(None),
        Some(s) => match s.to_str() {
            Some(ext) => Ok(Some(ext.to_string())),
            None => Err(FileError::os_str_err(s.to_os_string())),
        }
    }
}

// `a/b/c.d` -> `c.d`
pub fn basename(path: &str) -> Result<String, FileError> {
    let path_buf = PathBuf::from_str(path).expect("Internal Compiler Error 2CFFEEB12CD");  // it's infallible

    match path_buf.file_name() {
        None => Ok(String::new()),  // when the path terminates in `..`
        Some(s) => match s.to_str() {
            Some(ext) => Ok(ext.to_string()),
            None => Err(FileError::os_str_err(s.to_os_string())),
        }
    }
}

// `a/b/`, `c.d` -> `a/b/c.d`
pub fn join(path: &str, child: &str) -> Result<String, FileError> {
    let mut path_buf = PathBuf::from_str(path).expect("Internal Compiler Error EB4D870AEE0");  // Infallible
    let child = PathBuf::from_str(child).expect("Internal Compiler Error 25893E4A953");  // Infallible

    path_buf.push(child);

    match path_buf.to_str() {
        Some(result) => Ok(result.to_string()),
        None => Err(FileError::os_str_err(path_buf.into_os_string())),
    }
}

// `a/b/c.d, e` -> `a/b/c.e`
pub fn set_ext(path: &str, ext: &str) -> Result<String, FileError> {
    let mut path_buf = PathBuf::from_str(path).expect("Internal Compiler Error 34913906387");  // Infallible

    if path_buf.set_extension(ext) {
        match path_buf.to_str() {
            Some(result) => Ok(result.to_string()),
            None => Err(FileError::os_str_err(path_buf.into_os_string())),
        }
    } else {
        // has no filename
        Ok(path.to_string())
    }
}

pub fn is_dir(path: &str) -> bool {
    match PathBuf::from_str(path) {
        Err(_) => false,
        Ok(path) => path.is_dir(),
    }
}

pub fn exists(path: &str) -> bool {
    match PathBuf::from_str(path) {
        Err(_) => false,
        Ok(path) => path.exists(),
    }
}

pub fn read_dir(path: &str) -> Result<Vec<String>, FileError> {
    match fs::read_dir(path) {
        Err(e) => Err(FileError::init(e, path)),
        Ok(entries) => {
            let mut result = vec![];

            for entry in entries {
                match entry {
                    Err(e) => {
                        return Err(FileError::init(e, path));
                    }
                    Ok(e) => {
                        if let Some(ee) = e.path().to_str() {
                            result.push(ee.to_string());
                        }
                    }
                }
            }

            result.sort();
            Ok(result)
        }
    }
}

pub fn get_all_sdg(path: &str, recurs: bool, ext: &str) -> Result<Vec<String>, FileError> {
    let mut result = vec![];

    get_all_sdg_worker(path, recurs, ext, &mut result)?;

    Ok(result)
}

fn get_all_sdg_worker(path: &str, recurs: bool, ext: &str, buf: &mut Vec<String>) -> Result<(), FileError> {
    for file in read_dir(path)?.iter() {
        if is_dir(file) {
            if recurs {
                get_all_sdg_worker(file, recurs, ext, buf)?;
            }
        }

        else {
            match extension(file)? {
                Some(this_ext) if this_ext.to_lowercase() == ext => {
                    buf.push(file.to_string());
                },
                _ => {},
            }
        }
    }

    Ok(())
}
