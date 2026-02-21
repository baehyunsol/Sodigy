use sodigy_driver::CompileStage;
use sodigy_file::{FileOrStd, ModulePath};
use sodigy_span::Span;

mod command;
mod error;
mod ir_store;

pub use command::Command;
pub use error::Error;
pub use ir_store::{EmitIrOption, StoreIrAt, emit_irs_if_has_to, get_cached_ir};

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
