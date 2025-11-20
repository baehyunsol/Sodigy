use crate::{
    Backend,
    CompileStage,
    EmitIrOption,
    Optimization,
    Profile,
    StoreIrAt,
};
use sodigy_file::{FileOrStd, ModulePath};
use sodigy_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Command {
    InitIrDir {
        intermediate_dir: String,
    },
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
    // Run this after you have hir of all the modules.
    InterHir {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
    },
    // Run this after you have mir of all the modules.
    InterMir {
        modules: HashMap<ModulePath, Span>,
        intermediate_dir: String,
        stop_after: CompileStage,
        emit_ir_options: Vec<EmitIrOption>,
        dump_type_info: bool,
        output_path: Option<String>,
        backend: Backend,
        profile: Profile,
        optimization: Optimization,
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
