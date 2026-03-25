use crate::{FileOrStd, GetFilePathError, STD_FILES};
use sodigy_fs_api::{exists, join, join3, set_extension};
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

    pub fn get_file_path(&self, root: &str) -> Result<FileOrStd, GetFilePathError> {
        if self.is_lib() {
            // TODO: how about `src/lib/mod.sdg`?
            let p = join(root, "lib.sdg").unwrap();

            if exists(&p) {
                Ok(FileOrStd::File(p))
            } else {
                Err(GetFilePathError {
                    is_lib: true,
                    is_std: false,
                    module_path: self.clone(),
                    found_files: vec![],
                    candidates: vec![p],
                })
            }
        }

        else if self.is_std {
            let module_path = self.to_string();

            for (i, (module_path_, _, _, _)) in STD_FILES.iter().enumerate() {
                if module_path == *module_path_ {
                    return Ok(FileOrStd::Std(i as u32));
                }
            }

            panic!("TODO: {module_path:?}")
        }

        else {
            let joined = self.path.join("/");
            let candidate1 = join(
                root,
                &set_extension(&joined, "sdg").unwrap(),
            ).unwrap();
            let candidate2 = join3(
                root,
                &joined,
                "mod.sdg",
            ).unwrap();

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
        write!(
            fmt,
            "{}lib{}",
            if self.is_std { "@std." } else { "" },
            self.path.iter().map(
                |path| format!(".{path}")
            ).collect::<Vec<_>>().concat(),
        )
    }
}
