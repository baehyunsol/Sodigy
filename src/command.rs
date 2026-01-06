use crate::{
    CompileStage,
    EmitIrOption,
    Profile,
    StoreIrAt,
};
use sodigy_file::{FileOrStd, ModulePath};
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

        intermediate_dir: String,

        // When first generating hir, it has to find sub-modules in the module
        // so that the compiler can continue compiling. If it's using the cached
        // hir, it doesn't have to do so.
        find_modules: bool,

        emit_ir_options: Vec<EmitIrOption>,
        stop_after: CompileStage,
    },
    InterHir {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
    },
    InterMir {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
    },
    Interpret {
        bytecodes_path: StoreIrAt,

        // It's either `Test` or not.
        // The bytecode will tell you where the tests are, if exist, and where the
        // main function is, if exists. But it won't tell you how to optimize itself.
        profile: Profile,
    },
    Help(String),
}
