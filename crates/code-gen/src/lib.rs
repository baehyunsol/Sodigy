use sodigy_endec::Endec;
use sodigy_lir::Session;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    C,  // WIP
    Rust,  // WIP
    Python,  // WIP
    Bytecode,
}

pub fn lower(lir_session: Session, backend: Backend) -> Vec<u8> {
    match backend {
        Backend::Bytecode => {
            let executable = lir_session.into_executable();
            executable.encode()
        },
        _ => todo!(),
    }
}
