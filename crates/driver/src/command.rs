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

// Read `crates/driver/src/compile_stage.rs` for more information.
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
        dump_post_mir_log: bool,
        stop_after: CompileStage,
        validate_token_spans: ValidateTokenSpans,
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

        // It has nothing to do with the actual compilation.
        // It checks if the built_in funcs in the sodigy std and the
        // `mir::Intrinsic` match. If not, the compiler panics.
        verify_built_ins: bool,
    },
    // Collects per-module bytecodes and runs Link/CodeGen stage.
    // The result (bytecode or generated executable) is saved at `output_path`.
    CodeGen {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
        backend: Backend,
        dump_bytecodes: bool,
        output_path: StoreIrAt,
    },
    LoadInterHirSession {
        intermediate_dir: String,
    },
    LoadMirGlobalContext {
        intermediate_dir: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub enum ValidateTokenSpans {
    Never,
    OnlyStd,
    ExceptStd,
    Always,
}

impl ValidateTokenSpans {
    pub fn to_boolean(&self, is_std: bool) -> bool {
        match self {
            ValidateTokenSpans::Never => false,
            ValidateTokenSpans::OnlyStd => is_std,
            ValidateTokenSpans::ExceptStd => !is_std,
            ValidateTokenSpans::Always => true,
        }
    }
}
