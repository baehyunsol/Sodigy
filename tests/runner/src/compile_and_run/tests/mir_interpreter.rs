use super::{CnrContext, CompileAndRun};

impl CnrContext {
    pub fn mir_interpreter_test(&self, result: &CompileAndRun) -> Result<(), String> {
        // TODO: it ran with bytecode interpreter. run it again with mir interpreter,
        //       and make sure that their stdout are the same
        // TODO: what if the program is non-deterministic?
        Ok(())
    }
}
