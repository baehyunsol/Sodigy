use sodigy_bytecode::{self as bytecode, ObjectFile};
use sodigy_endec::Endec;
use sodigy_error::{Error, Warning};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    Exe,  // WIP
    ReadableBytecode,
    ExecutableBytecode,

    // Emit::C emits a single C file, while Emit::MultiC emits
    // multiple C files and a system-C-compiler should compile and link them.
    C,  // WIP
    MultiC,  // WIP
}

pub fn lower(
    object_files: Vec<ObjectFile>,
    errors: Vec<Error>,
    warnings: Vec<Warning>,
    emit: Emit,
) -> (Vec<u8>, Vec<Error>, Vec<Warning>) {
    match emit {
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
        _ => todo!(),
    }
}
