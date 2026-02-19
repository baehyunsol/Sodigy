use crate::{
    CompileStage,
    EmitIrOption,
    StoreIrAt,
};
use sodigy_code_gen::Backend;
use sodigy_file::{FileOrStd, ModulePath};
use sodigy_optimize::OptimizeLevel;
use sodigy_span::Span;
use std::collections::HashMap;

// Read `src/compile_stage.rs` for more information.
#[derive(Clone, Debug)]
pub enum Command {
    PerFileIr {
        // A module is (almost always) a file.
        // A module `foo/bar` can be found in either `src/foo/bar.sdg` or `src/foo/bar/mod.sdg`.
        input_file_path: FileOrStd,
        input_module_path: ModulePath,
        optimize_level: OptimizeLevel,

        intermediate_dir: String,

        // When first generating hir, it has to find sub-modules in the module
        // so that the compiler can continue compiling. If it's using the cached
        // hir, it doesn't have to do so.
        find_modules: bool,

        emit_ir_options: Vec<EmitIrOption>,
        stop_after: CompileStage,
    },
    // Collects HIRs and runs InterHir stage.
    InterHir {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
        emit_ir_options: Vec<EmitIrOption>,
    },
    // Collects MIRs and runs InterMir stage.
    InterMir {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
        emit_ir_options: Vec<EmitIrOption>,
    },
    // Collects per-module bytecodes and runs Link/CodeGen stage.
    // The result (bytecode or generated executable) is saved at `output_path`.
    CodeGen {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
        backend: Backend,
        output_path: StoreIrAt,
    },
}
