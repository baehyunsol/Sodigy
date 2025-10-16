use sodigy_backend::CodeGenMode;

mod cli;

pub use cli::{Command, parse_args};

#[derive(Clone, Copy, Debug)]
pub enum IrKind {
    Code,
    Ast,
    Hir,
    Mir,
    Bytecode,
}

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    C,
    Rust,
    Python,
    Bytecode,
}

#[derive(Clone, Copy, Debug)]
pub enum Profile {
    Debug,
    Release,
    Test,
}

impl From<Profile> for CodeGenMode {
    fn from(p: Profile) -> CodeGenMode {
        match p {
            Profile::Debug => CodeGenMode::Binary,
            Profile::Release => CodeGenMode::Binary,
            Profile::Test => CodeGenMode::Test,
        }
    }
}
