use sodigy_bytecode::{self as bytecode, ObjectFile, Session};
use sodigy_endec::Endec;
use sodigy_error::{Error, Warning};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    C,  // WIP
    Rust,  // WIP
    Python,  // WIP
    Bytecode,
}

pub fn lower(
    object_files: Vec<ObjectFile>,
    errors: Vec<Error>,
    warnings: Vec<Warning>,
    backend: Backend,
) -> (Vec<u8>, Vec<Error>, Vec<Warning>) {
    match backend {
        // It doesn't generate extra errors/warnings!
        Backend::Bytecode => (
            bytecode::flatten(&bytecode::link(object_files)).encode(),
            errors,
            warnings,
        ),
        _ => todo!(),
    }
}
