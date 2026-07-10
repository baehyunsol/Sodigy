use super::{CnrContext, CompileAndRun};
use crate::subprocess;

impl CnrContext {
    pub fn optimization_test(&self, result: &CompileAndRun) -> Result<(), String> {
        // TODO: I want some cases to be cleaned, and not the others.
        self.clean()?;

        match subprocess::run(
            &self.sodigy_path,
            &["test", "--release", "--emit-irs"],
            &self.project_dir,
            30.0,
            false,
            false,
        ) {
            Ok(output) => {
                if output.code() != Some(0) {
                    Err(format!(
                        "Failed to compile or run the code with optimization{}",
                        if self.dump_output {
                            format!(":\n{}", String::from_utf8_lossy(&output.stderr))
                        } else {
                            String::from(".")
                        },
                    ))
                }

                else if &output.stdout != result.stdout.as_bytes() {
                    Err(format!(
                        "Optimized and unoptimized program have different stdout:\nunoptimized: {:?}\noptimized: {:?}",
                        String::from_utf8_lossy(&output.stdout),
                        result.stdout,
                    ))
                }

                else {
                    Ok(())
                }
            },
            Err(e) => Err(format!("error with `sodigy test --release`: {e:?}")),
        }
    }
}
