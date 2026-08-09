use super::{CnrContext, CompileAndRun};
use crate::subprocess;

impl CnrContext {
    pub fn optimization_test(&self, result: &CompileAndRun) -> Result<(), String> {
        // We have to test both cases: with/without `sodigy clean` before
        // compiling the sample with optimization.
        // So, half of the samples are run with `sodigy clean` and the others are not.
        if self.cnr_seq % 2 == 0 {
            self.clean()?;
        }

        let mut args = vec!["test", "--release"];

        if self.emit_irs {
            args.push("--emit-irs");
        }

        match subprocess::run(
            &self.sodigy_path,
            &args,
            &self.project_dir,
            30.0,
            false,  // dump_output
            false,  // check_nonzero_status
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
