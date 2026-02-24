use super::{CnrContext, CompileAndRun};

impl CnrContext {
    pub fn optimization_test(&self, result: &CompileAndRun) -> Result<(), String> {
        // TODO: run it with/without optimization and make sure that their stdout are the same
        // TODO: what if the program is non-deterministic?
        Ok(())
    }
}
