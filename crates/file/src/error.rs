use crate::ModulePath;

#[derive(Clone, Debug)]
pub struct GetFilePathError {
    pub is_lib: bool,
    pub is_std: bool,
    pub module_path: ModulePath,
    pub candidates: Vec<String>,
    pub found_files: Vec<String>,
}
