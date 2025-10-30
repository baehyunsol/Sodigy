use sodigy_backend::CodeGenMode;

mod cli;
mod command;
mod error;

pub use cli::{CliCommand, parse_args};
pub use command::Command;
pub use error::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompileStage {
    Lex,
    Parse,
    Hir,
    InterHir,
    Mir,
    TypeCheck,
    Bytecode,
    CodeGen,
}

impl CompileStage {
    pub fn all() -> Vec<CompileStage> {
        vec![
            CompileStage::Lex,
            CompileStage::Parse,
            CompileStage::Hir,
            CompileStage::InterHir,
            CompileStage::Mir,
            CompileStage::TypeCheck,
            CompileStage::Bytecode,
            CompileStage::CodeGen,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    C,
    Rust,
    Python,
    Bytecode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Script,
    Test,
}

// TODO: just use the same enum...
impl From<Profile> for CodeGenMode {
    fn from(p: Profile) -> CodeGenMode {
        match p {
            Profile::Script => CodeGenMode::Binary,
            Profile::Test => CodeGenMode::Test,
        }
    }
}

/// The compiler stores irs (or result) in various places.
/// 1. It can store the output to user-given path.
/// 2. If it has to interpret the bytecodes, it just stores them in memory and directly executes them.
/// 3. In a complicated compilation process, it stores irs in the intermediate_dir.
#[derive(Clone, Debug)]
pub enum StoreIrAt {
    File(String),
    Memory,
    IntermediateDir,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Optimization {
    None,
    Mild,
    Extreme,
}

#[derive(Clone, Debug)]
pub struct EmitIrOption {
    pub stage: CompileStage,
    pub store: StoreIrAt,
    pub human_readable: bool,
}
