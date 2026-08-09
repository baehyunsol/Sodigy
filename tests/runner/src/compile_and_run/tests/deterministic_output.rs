use super::{CnrContext, CompileAndRun, Status};
use sodigy_fs_api::{FileError, exists, join4, read_string};

impl CnrContext {
    pub fn deterministic_output_test(&self, result: &CompileAndRun) -> Result<(), String> {
        self.clean()?;

        // step 0
        match self.get_bytecode_output() {
            Ok(None) => {},
            Ok(Some(_)) => {
                return Err(String::from("failed at step 0\nran `sodigy clean`, but the bytecode output is not cleaned"));
            },
            Err(e) => {
                return Err(format!("failed at step 0\n{e:?}"));
            },
        }

        // step 1. fresh-build without optimization
        self.run_sodigy(false, Status::RunPass)?;

        let bytecode1 = match self.get_bytecode_output() {
            Ok(Some(b)) => b,
            Ok(None) => {
                return Err(String::from("failed at step 1\ncannot find the bytecode output"));
            },
            Err(e) => {
                return Err(format!("failed at step 1\n{e:?}"));
            },
        };

        // step 2. cached-build without optimization
        self.run_sodigy(false, Status::RunPass)?;

        let bytecode2 = match self.get_bytecode_output() {
            Ok(Some(b)) => b,
            Ok(None) => {
                return Err(String::from("failed at step 2\ncannot find the bytecode output"));
            },
            Err(e) => {
                return Err(format!("failed at step 2\n{e:?}"));
            },
        };

        // step 3. fresh-build without optimization
        self.clean()?;
        self.run_sodigy(false, Status::RunPass)?;

        let bytecode3 = match self.get_bytecode_output() {
            Ok(Some(b)) => b,
            Ok(None) => {
                return Err(String::from("failed at step 3\ncannot find the bytecode output"));
            },
            Err(e) => {
                return Err(format!("failed at step 3\n{e:?}"));
            },
        };

        // TODO: show diffs!!
        if bytecode1 != bytecode2 {
            return Err(String::from("bytecode1 != bytecode2"));
        }

        if bytecode2 != bytecode3 {
            return Err(String::from("bytecode2 != bytecode3"));
        }

        // step 4. fresh-build with optimization
        self.clean()?;
        self.run_sodigy(true, Status::RunPass)?;

        let bytecode4 = match self.get_bytecode_output() {
            Ok(Some(b)) => b,
            Ok(None) => {
                return Err(String::from("failed at step 4\ncannot find the bytecode output"));
            },
            Err(e) => {
                return Err(format!("failed at step 4\n{e:?}"));
            },
        };

        // step 5. fresh-build with optimization
        self.clean()?;
        self.run_sodigy(true, Status::RunPass)?;

        let bytecode5 = match self.get_bytecode_output() {
            Ok(Some(b)) => b,
            Ok(None) => {
                return Err(String::from("failed at step 5\ncannot find the bytecode output"));
            },
            Err(e) => {
                return Err(format!("failed at step 5\n{e:?}"));
            },
        };

        if bytecode4 != bytecode5 {
            return Err(String::from("bytecode4 != bytecode5"));
        }

        Ok(())
    }

    fn get_bytecode_output(&self) -> Result<Option<String>, FileError> {
        let bytecode_output_at = join4(
            &self.project_dir,
            "target",
            "irs",
            "bytecodes",
        )?;

        if !exists(&bytecode_output_at) {
            return Ok(None);
        } else {
            return Ok(Some(read_string(&bytecode_output_at)?));
        }
    }
}
