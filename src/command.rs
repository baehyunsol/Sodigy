use crate::{
    Backend,
    IrKind,
    IrStore,
    Optimization,
    Profile,
};

#[derive(Clone, Debug)]
pub enum Command {
    InitIrDir {
        intermediate_dir: String,
    },
    Compile {
        input_path: String,
        input_kind: IrKind,
        intermediate_dir: String,
        reuse_ir: bool,

        // These two are for debugging the compiler.
        // I'll make a CLI option for these, someday.
        emit_irs: bool,
        dump_type_info: bool,

        output_path: IrStore,
        output_kind: IrKind,
        backend: Backend,
        profile: Profile,
        optimization: Optimization,
    },
    Interpret {
        bytecodes_path: IrStore,

        // It's either `Test` or not.
        // The bytecode will tell you where the tests are, if exist, and where the
        // main function is, if exists. But it won't tell you how to optimize itself.
        profile: Profile,
    },
    Help(String),
}
