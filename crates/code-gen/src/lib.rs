use sodigy_bytecode::Session;
use sodigy_endec::Endec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    C,  // WIP
    Rust,  // WIP
    Python,  // WIP
    Bytecode,
}

pub fn lower(bytecode_session: Session, backend: Backend) -> Vec<u8> {
    match backend {
        Backend::Bytecode => {
            let executable = bytecode_session.into_executable();
            executable.encode()
        },
        _ => todo!(),
    }
}
