// #![allow(dead_code)]
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

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
            _ => panic!("e: {e:?}, path: {given_path}"),
        };

        FileError {
            kind, given_path: Some(given_path.to_string())
        }
    }

    pub fn os_str_err(os_str: OsString) -> Self {
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

// `a/b/c.d -> `c``
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

// `a/b/c.d -> `d``
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

use std::collections::HashMap;
static mut READ_DIR_CACHE: *mut HashMap<String, Vec<String>> = std::ptr::null_mut();
static mut READ_DIR_CACHE_INIT: bool = false;

pub unsafe fn init_dir_cache() {
    if READ_DIR_CACHE_INIT {
        return;
    }

    let mut c = HashMap::new();
    READ_DIR_CACHE = &mut c as *mut HashMap<_, _>;
    std::mem::forget(c);
    READ_DIR_CACHE_INIT = true;
}

// use this func only when the `path` is read-only.
pub fn read_dir_cached(path: &str) -> Result<Vec<String>, FileError> {
    unsafe {
        if !READ_DIR_CACHE_INIT {
            init_dir_cache();
            READ_DIR_CACHE_INIT = true;
        }

        if let Some(v) = READ_DIR_CACHE.as_mut().unwrap().get(path) {
            return Ok(v.to_vec());
        }

        let result = read_dir(path)?;
        READ_DIR_CACHE.as_mut().unwrap().insert(path.to_string(), result.clone());

        Ok(result)
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

pub fn make_dir(path: &str) -> Result<(), FileError> {

    match fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            io::ErrorKind::PermissionDenied => Err(FileError::init(e, path)),
            _ => Ok(()),
        },
    }

}

pub fn remove_dir(path: &str) -> Result<(), FileError> {
    let ls = match read_dir(path) {
        Ok(ls) => ls,
        Err(e) => match e.kind {
            FileErrorKind::FileNotFound => {
                return Ok(());  // already removed :)
            },
            _ => {
                return Err(e);
            }
        },
    };

    for fd in ls.iter() {
        if is_dir(fd) {
            remove_dir(fd)?;
        }

        else {
            match fs::remove_file(fd) {
                Err(e) => {
                    return Err(FileError::init(e, fd));
                }
                _ => {}
            }
        }
    }

    match fs::remove_dir(path) {
        Err(e) => Err(FileError::init(e, path)),
        _ => Ok(()),
    }
}

pub fn get_sub_directories_recursive(path: &str) -> Vec<String> {

    match read_dir(path) {
        Err(_) => vec![],
        Ok(files) => {
            let sub_dirs = files.into_iter().filter(|f| is_dir(f)).collect::<Vec<String>>();

            let sub_sub = sub_dirs.iter().map(|dir| get_sub_directories_recursive(dir)).collect::<Vec<Vec<String>>>().concat();

            vec![
                sub_dirs,
                sub_sub
            ].concat()
        }
    }

}
