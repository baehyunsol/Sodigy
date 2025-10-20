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

mod file_map;
mod fmt;

use file_map::{
    length_file_map,
    push_file_map,
    search_file_map,
    search_file_map_by_id,
};

// It represents a file in a project.
//
// Its `Ord` is for deterministic output of the error messages (it sorts the errors by file).
// It doesn't sort the files by name.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct File {
    // If it's compiling multiple projects, the compiler gives sequential numbers.
    // The top-level project is always 0.
    pub project: u32,

    // If there are multiple files in the project, the compiler gives sequential numbers.
    pub file: u32,
}

impl File {
    pub fn register(
        project_id: u32,

        // `read_bytes(path)` should work
        path: &str,

        // whatever string that can uniquely identify this file
        normalized_path: &str,

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

        // file_map is a list of `file_id: u32`, `content_hash: u128`, `normalized_path: String`
        let (mut file_map, file) = if exists(&file_map_path) {
            let file_map = read_bytes(&file_map_path)?;

            match search_file_map(&file_map, normalized_path, &file_map_path)? {
                // If it's already registered, it returns the previous one without updating its content_hash.
                // That means you cannot update a file while a compilation is going on.
                Some((file_id, _)) => {
                    return Ok(File {
                        project: project_id,
                        file: file_id,
                    });
                },
                None => {
                    let file_id = length_file_map(&file_map, &file_map_path)? as u32;
                    (
                        file_map,
                        File {
                            project: project_id,
                            file: file_id,
                        },
                    )
                },
            }
        } else {
            (
                vec![],
                File {
                    project: project_id,
                    // This is the first file!
                    file: 0,
                },
            )
        };

        let content = read_bytes(path)?;
        let content_hash = intern_string(&content, intermediate_dir)?;
        push_file_map(&mut file_map, file.file, content_hash.0, normalized_path);
        write_bytes(
            &join(
                intermediate_dir,
                &format!("file_{project_id}"),
            )?,
            &file_map,
            WriteMode::CreateOrTruncate,
        )?;
        lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;
        Ok(file)
    }

    // This is very very expensive.
    pub fn get_path(&self, intermediate_dir: &str) -> Result<Option<String>, FileError> {
        let project_id = self.project;

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

        match search_file_map_by_id(&file_map, self.file, &file_map_path)? {
            Some((path, _)) => Ok(Some(path.to_string())),
            None => Ok(None),
        }
    }

    // This is very very expensive.
    pub fn read_bytes(&self, intermediate_dir: &str) -> Result<Option<Vec<u8>>, FileError> {
        let project_id = self.project;

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

        match search_file_map_by_id(&file_map, self.file, &file_map_path)? {
            Some((_, content_hash)) => unintern_string(InternedString(content_hash), intermediate_dir),
            None => Ok(None),
        }
    }
}
