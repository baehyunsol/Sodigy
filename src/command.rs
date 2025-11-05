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
    Compile {
        // A module is (almost always) a file.
        // A module `foo/bar` can be found in either `src/foo/bar.sdg` or `src/foo/bar/mod.sdg`.
        input_file_path: FileOrStd,
        input_module_path: ModulePath,

        intermediate_dir: String,
        emit_ir_options: Vec<EmitIrOption>,

        // It's for debugging the compiler.
        // I'll make a CLI option for this, someday.
        dump_type_info: bool,

        // You can quit termination after emitting irs.
        output_path: Option<String>,

        stop_after: CompileStage,
        backend: Backend,
        profile: Profile,
        optimization: Optimization,
    },
    InterHir {
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
