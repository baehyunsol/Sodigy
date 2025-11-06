use crate::std_file::STD_FILES;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    exists,
    join,
    read_bytes,
    write_bytes,
};
use sodigy_string::{InternedString, intern_string, unintern_string};
use std::fs::File as StdFile;

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
    search_content_hashes_by_module_paths,
    search_file_map_by_id,
    search_file_map_by_module_path,
};

// It represents a file in a project.
//
// Its `Ord` is for deterministic output of the error messages (it sorts the errors by file).
// It doesn't sort the files by name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum File {
    File {
        // If it's compiling multiple projects, the compiler gives sequential numbers.
        // The top-level project is always 0.
        project: u32,

        // If there are multiple files in the project, the compiler gives sequential numbers.
        file: u32,
    },
    Std(u64),
}

impl File {
    pub fn clear_cache(project_id: u32, intermediate_dir: &str) -> Result<(), FileError> {
        let lock_file_path = join(
            intermediate_dir,
            &format!("file_map_{project_id}_lock"),
        )?;
        let lock_file = StdFile::create(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
        lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

        let file_map_path = join(
            intermediate_dir,
            &format!("files_{project_id}"),
        )?;
        write_bytes(&file_map_path, b"", WriteMode::CreateOrTruncate)?;
        lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;
        Ok(())
    }

    pub fn register(
        project_id: u32,

        // `read_bytes(file_path)` should work
        file_path: &str,

        // Each module has a unique module_path within a project.
        // It's like file_path, but represents a module hierarchy.
        // For example, a module with `foo/bar` can be in `src/foo/bar.rs` or `src/foo/bar/mod.rs`.
        module_path: &str,

        intermediate_dir: &str,
    ) -> Result<File, FileError> {
        let lock_file_path = join(
            intermediate_dir,
            &format!("file_map_{project_id}_lock"),
        )?;
        let lock_file = StdFile::create(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
        lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

        let file_map_path = join(
            intermediate_dir,
            &format!("files_{project_id}"),
        )?;

        // file_map is a list of `file_id: u32`, `content_hash: u128`, `module_path: String`
        let (mut file_map, file, file_id) = if exists(&file_map_path) {
            let file_map = read_bytes(&file_map_path)?;

            match search_file_map_by_module_path(&file_map, module_path, &file_map_path)? {
                // If it's already registered, it returns the previous one without updating its content_hash.
                // That means you cannot update a file while a compilation is going on.
                Some((file_id, _)) => {
                    return Ok(File::File {
                        project: project_id,
                        file: file_id,
                    });
                },
                None => {
                    let file_id = length_file_map(&file_map, &file_map_path)? as u32;
                    (
                        file_map,
                        File::File {
                            project: project_id,
                            file: file_id,
                        },
                        file_id,
                    )
                },
            }
        } else {
            (
                vec![],
                File::File {
                    project: project_id,
                    // This is the first file!
                    file: 0,
                },
                0,
            )
        };

        let content = read_bytes(file_path)?;
        let content_hash = intern_string(&content, intermediate_dir)?;
        push_file_map(&mut file_map, file_id, content_hash.0, module_path, file_path);
        write_bytes(
            &join(
                intermediate_dir,
                &format!("files_{project_id}"),
            )?,
            &file_map,
            WriteMode::CreateOrTruncate,
        )?;
        lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;
        Ok(file)
    }

    pub fn from_module_path(project_id: u32, path: &str, intermediate_dir: &str) -> Result<Option<File>, FileError> {
        let lock_file_path = join(
            intermediate_dir,
            &format!("file_map_{project_id}_lock"),
        )?;
        let lock_file = StdFile::create(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
        lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

        let file_map_path = join(
            intermediate_dir,
            &format!("files_{project_id}"),
        )?;
        let file_map = read_bytes(&file_map_path)?;

        lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

        match search_file_map_by_module_path(&file_map, path, &file_map_path)? {
            Some((file_id, _)) => Ok(Some(File::File { project: project_id, file: file_id })),
            None => Ok(None),
        }
    }

    // It returns (module_path, file_path).
    // This is very very expensive.
    pub fn get_path(&self, intermediate_dir: &str) -> Result<Option<(String, String)>, FileError> {
        match self {
            File::File { project: project_id, file: file_id } => {
                let lock_file_path = join(
                    intermediate_dir,
                    &format!("file_map_{project_id}_lock"),
                )?;
                let lock_file = StdFile::create(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
                lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

                let file_map_path = join(
                    intermediate_dir,
                    &format!("files_{project_id}"),
                )?;
                let file_map = read_bytes(&file_map_path)?;

                lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

                match search_file_map_by_id(&file_map, *file_id, &file_map_path)? {
                    Some((module_path, file_path, _)) => Ok(Some((module_path.to_string(), file_path.to_string()))),
                    None => Ok(None),
                }
            },
            File::Std(id) => Ok(Some((
                STD_FILES[*id as usize].0.to_string(),
                STD_FILES[*id as usize].1.to_string(),
            ))),
        }
    }

    pub fn get_content_hash(&self, intermediate_dir: &str) -> Result<u128, FileError> {
        match self {
            File::File { project: project_id, file: file_id } => {
                let lock_file_path = join(
                    intermediate_dir,
                    &format!("file_map_{project_id}_lock"),
                )?;
                let lock_file = StdFile::create(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
                lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

                let file_map_path = join(
                    intermediate_dir,
                    &format!("files_{project_id}"),
                )?;
                let file_map = read_bytes(&file_map_path)?;

                lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

                match search_file_map_by_id(&file_map, *file_id, &file_map_path)? {
                    Some((_, _, content_hash)) => Ok(content_hash),

                    // error? panic? unreachable?
                    None => todo!(),
                }
            },
            File::Std(id) => Ok(STD_FILES[*id as usize].3),
        }
    }

    // This is very very expensive.
    pub fn read_bytes(&self, intermediate_dir: &str) -> Result<Option<Vec<u8>>, FileError> {
        match self {
            File::File { .. } => {
                let content_hash = self.get_content_hash(intermediate_dir)?;
                Ok(unintern_string(InternedString(content_hash), intermediate_dir)?)
            },
            File::Std(id) => Ok(Some(STD_FILES[*id as usize].2.to_vec())),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FileOrStd {
    File(String),
    Std(u64),
}
