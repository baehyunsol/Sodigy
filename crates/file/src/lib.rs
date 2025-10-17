use sodigy_fs_api::FileError;

mod fmt;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum File {
    // If the compiler is dealing with only one file, it just doesn't care.
    // For example, hir is created per-file, so it gives `File::Single` to all the spans.
    // Later, when the compiler has to do inter-file analysis, the compiler gives different `File` to the spans.
    Single,
}

impl File {
    pub fn get_name(&self) -> String {
        todo!()
    }

    pub fn read_bytes(&self) -> Result<Vec<u8>, FileError> {
        todo!()
    }
}
