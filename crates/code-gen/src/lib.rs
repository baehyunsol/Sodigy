use sodigy_endec::Endec;
use sodigy_error::{Error, Warning};
use sodigy_object_file::{self as object_file, ObjectFile};

mod rust;

#[derive(Clone, Debug)]
pub enum Profile {
    Run,
    Test {
        std_assertions: bool,
        filter: Option<Vec<Filter>>,
    },
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub keyword: String,
    pub match_start: bool,  // `^`
    pub match_end: bool,  // `$`
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    Exe,  // WIP
    ReadableBytecode,
    ExecutableBytecode,
    Rust,  // WIP
}

pub fn lower(
    object_files: Vec<ObjectFile>,
    profile: Profile,
    mut errors: Vec<Error>,
    mut warnings: Vec<Warning>,
    emit: Emit,
) -> (Vec<u8>, Vec<Error>, Vec<Warning>) {
    match emit {
        Emit::Exe => todo!(),
        Emit::ReadableBytecode => (
            object_file::link(object_files).to_string().into_bytes(),
            errors,
            warnings,
        ),
        Emit::ExecutableBytecode => (
            object_file::link(object_files).encode(),
            errors,
            warnings,
        ),
        Emit::Rust => {
            let code = rust::lower(
                object_file::link(object_files),
                profile,
                &mut errors,
                &mut warnings,
            ).code.into_bytes();

            (code, errors, warnings)
        },
    }
}

