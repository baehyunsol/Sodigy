use crate::Profile;
use sodigy_bytecode::ObjectFile;

pub struct CModule {
    pub code: String,
    pub header: String,
}

pub fn lower(object_file: ObjectFile, profile: Profile) -> CModule {
    todo!()
}
