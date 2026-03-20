use super::{CnrContext, CompileAndRun, LineMatcher, Status, hash_dir, match_lines};
use crate::subprocess;

mod incremental_compilation;
mod main_test;
mod mir_interpreter;
mod optimization;
mod type_switch;

pub use main_test::{Directive, ExpectedOutput};

impl CnrContext {
    pub fn extra_tests(&self, result: &mut CompileAndRun) {
        if result.error.is_none() && result.status == Status::RunPass && self.sdg_files >= 3 {
            if let Err(e) = self.incremental_compilation_test(&result) {
                result.error = Some(format!("incremental compilation test fail\n\n{e}"));
            }
        }

        if result.error.is_none() && result.status == Status::RunPass {
            if let Err(e) = self.optimization_test(&result) {
                result.error = Some(format!("optimization test fail\n\n{e}"));
            }
        }

        if result.error.is_none() && result.status == Status::RunPass {
            if let Err(e) = self.mir_interpreter_test(&result) {
                result.error = Some(format!("mir interpreter test fail\n\n{e}"));
            }
        }

        if result.error.is_none() && (
            result.status == Status::CompilePass ||
            result.status == Status::RunTimeout ||
            result.status == Status::RunFail ||
            result.status == Status::RunPass
        ) {
            if let Err(e) = self.type_switch_test(&result) {
                result.error = Some(format!("type switch test fail\n\n{e}"));
            }
        }
    }

    pub fn clean(&self) -> Result<(), String> {
        match subprocess::run(
            &self.sodigy_path,
            &["clean"],
            &self.project_dir,
            5.0,
            false,
            false,
        ) {
            Ok(output) if !output.success() => Err(format!("error with `sodigy clean` (exit status {:?})", output.code())),
            Err(e) => Err(format!("error with `sodigy clean`: {e:?}")),
            Ok(_) => Ok(()),
        }
    }

    pub fn run_sodigy(&self, expected_result: Status) -> Result<(), String> {
        assert!(
            expected_result == Status::RunPass ||
            expected_result == Status::CompilePass ||
            expected_result == Status::RunFail ||
            expected_result == Status::CompileFail
        );

        // I use `sodigy test` command instead of `sodigy build` + `sodigy interpret` and that's intentional.
        //
        // The main test runner uses `sodigy build` + `sodigy interpret` and I want to test another path.
        let mut args = vec!["test"];

        if self.log_post_mir {
            args.push("--log-post-mir");
        }

        match subprocess::run(
            &self.sodigy_path,
            &args,
            &self.project_dir,
            30.0,
            false,
            false,
        ) {
            Ok(output) => match (output.code(), expected_result) {
                (Some(0), Status::RunPass) => Ok(()),
                (Some(10), Status::CompilePass | Status::RunFail) => Ok(()),
                (Some(11), Status::CompileFail) => Ok(()),
                _ => Err(format!("sodigy didn't run as expected! expect: {expected_result:?}, got: {:?}", output.code())),
            },
            Err(e) => Err(format!("error with `sodigy run`: {e:?}")),
        }
    }
}

#[derive(Clone, Copy)]
enum AnsiParseState {
    Text,
    Escape,
}

pub fn remove_ansi_characters(s: &str) -> String {
    let mut state = AnsiParseState::Text;
    let mut result = vec![];

    for ch in s.chars() {
        match state {
            AnsiParseState::Text => match ch {
                '\x1b' => {
                    state = AnsiParseState::Escape;
                },
                _ => {
                    result.push(ch);
                },
            },
            AnsiParseState::Escape => match ch {
                'm' => {
                    state = AnsiParseState::Text;
                },
                _ => {},
            },
        }
    }

    result.iter().collect()
}
