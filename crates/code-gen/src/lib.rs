use sodigy_bytecode::Session;
use sodigy_endec::Endec;
use sodigy_error::{Error, Warning};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    C,  // WIP
    Rust,  // WIP
    Python,  // WIP
    Bytecode,
}

pub fn lower(mut bytecode_session: Session, backend: Backend) -> (Vec<u8>, Vec<Error>, Vec<Warning>) {
    match backend {
        Backend::Bytecode => {
            let executable = bytecode_session.into_executable();

            // It doesn't generate extra errors/warnings!
            (
                executable.encode(),
                bytecode_session.errors.drain(..).collect(),
                bytecode_session.warnings.drain(..).collect(),
            )
        },
        _ => todo!(),
    }
}
