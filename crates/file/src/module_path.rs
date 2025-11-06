use crate::{FileOrStd, GetFilePathError};
use sodigy_fs_api::exists;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModulePath {
    pub(crate) path: Vec<String>,
    pub(crate) is_std: bool,
}

impl ModulePath {
    pub fn lib() -> ModulePath {
        ModulePath { path: vec![], is_std: false }
    }

    pub fn is_lib(&self) -> bool {
        self.path.is_empty() && !self.is_std
    }

    #[must_use = "method returns a new value and does not mutate the original value"]
    pub fn join(&self, module: String) -> ModulePath {
        let mut path = self.path.clone();
        path.push(module);
        ModulePath { path, is_std: self.is_std }
    }

    pub fn get_file_path(&self) -> Result<FileOrStd, GetFilePathError> {
        if self.is_lib() {
            // TODO: how about `src/lib/mod.sdg`?
            if exists("src/lib.sdg") {
                Ok(FileOrStd::File(String::from("src/lib.sdg")))
            } else {
                Err(GetFilePathError {
                    is_lib: true,
                    is_std: false,
                    module_path: self.clone(),
                    found_files: vec![],
                    candidates: vec![String::from("src/lib.sdg")],
                })
            }
        }

        else if self.is_std {
            todo!()
        }

        else {
            let joined = self.to_string();
            let candidate1 = format!("src/{}.sdg", joined.replace(".", "/"));
            let candidate2 = format!("src/{}/mod.sdg", joined.replace(".", "/"));

            match (exists(&candidate1), exists(&candidate2)) {
                (true, true) => Err(GetFilePathError {
                    is_lib: false,
                    is_std: false,
                    module_path: self.clone(),
                    candidates: vec![candidate1.clone(), candidate2.clone()],
                    found_files: vec![candidate1, candidate2],
                }),
                (false, false) => Err(GetFilePathError {
                    is_lib: false,
                    is_std: false,
                    module_path: self.clone(),
                    candidates: vec![candidate1, candidate2],
                    found_files: vec![],
                }),
                (true, false) => Ok(FileOrStd::File(candidate1)),
                (false, true) => Ok(FileOrStd::File(candidate2)),
            }
        }
    }
}

impl fmt::Display for ModulePath {
    /// Unique (in the project) identifier of this module.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "lib.{}", self.path.join("."))
    }
}
