use sodigy_file::{FileOrStd, ModulePath};
use sodigy_span::Span;

mod cli;
mod command;
mod compile_stage;
mod error;

pub use cli::{CliCommand, ColorWhen, parse_args};
pub use command::Command;
pub use compile_stage::{CompileStage, COMPILE_STAGES};
pub use error::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Script,
    Test,
}

/// The compiler stores irs (or result) in various places.
/// 1. It can store the output to user-given path.
/// 2. If it has to interpret the bytecodes, it just stores them in memory and directly executes them.
/// 3. In a complicated compilation process, it stores irs in the intermediate_dir.
#[derive(Clone, Debug)]
pub enum StoreIrAt {
    File(String),
    IntermediateDir,
}

#[derive(Clone, Debug)]
pub struct EmitIrOption {
    pub stage: CompileStage,
    pub store: StoreIrAt,
    pub human_readable: bool,
}

// The compiler compiles a project module-by-module. This is the status
// of each module's compilation.
//
// If `path` is `foo.sdg`, `compile_stage` is `Hir` and `running` is `false`,
// hir for `foo.sdg` is complete and no worker is working on this module.
// If `path` is `foo.sdg`, `compile_stage` is `Hir` and `running` is `true`,
// 1 worker is working on this module and when the worker is done, hir for
// the module will be complete.
#[derive(Clone, Debug)]
pub struct ModuleCompileState {
    pub module_path: ModulePath,
    pub file_path: FileOrStd,
    pub span: Span,
    pub compile_stage: CompileStage,
    pub running: bool,
}
