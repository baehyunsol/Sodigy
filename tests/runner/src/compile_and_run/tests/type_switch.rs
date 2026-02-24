use super::{CnrContext, CompileAndRun};

impl CnrContext {
    pub fn type_switch_test(&self, result: &CompileAndRun) -> Result<(), String> {
        // TODO
        //
        // It assumes that the program ran successfully before type_switch_test.
        //
        // Step 1.
        //   It converts some type annotations into type assertions, with the same type, and run
        //   the program again.
        //   1. If it runs successfully, that's good.
        //   2. If it throws e-0420 or e-0425, that's fine.
        //   3. If it throws another kind of error, that's not fine.
        // Step 2.
        //   It converts some type annotation into type assertions, with an obviously wrong type, and
        //   run the program again.
        //   1. If it runs successfully, that's not fine.
        //   2. If it throws e-0415, that's fine.
        //   3. If it throws another kind of error, that's not fine.
        Ok(())
    }
}
