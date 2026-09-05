use sodigy_bytecode::{self as bytecode, ObjectFile};
use sodigy_endec::Endec;
use sodigy_error::{Error, Warning};

mod c;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Run,
    Test,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    Exe,  // WIP
    ReadableBytecode,
    ExecutableBytecode,
    C,  // WIP
}

pub fn lower(
    object_files: Vec<ObjectFile>,
    profile: Profile,
    errors: Vec<Error>,
    warnings: Vec<Warning>,
    emit: Emit,
) -> (Vec<u8>, Vec<Error>, Vec<Warning>) {
    match emit {
        Emit::Exe => todo!(),
        Emit::ReadableBytecode => (
            bytecode::link(object_files).to_string().into_bytes(),
            errors,
            warnings,
        ),
        Emit::ExecutableBytecode => (
            bytecode::flatten(&mut bytecode::link(object_files)).encode(),
            errors,
            warnings,
        ),
        Emit::C => (
            c::lower(
                bytecode::link(object_files),
                profile,
            ).code.into_bytes(),
            errors,
            warnings,
        ),
    }
}
