use super::{CnrContext, CompileAndRun, Status};
use crate::subprocess;
use sodigy_fs_api::{join, read_bytes};

#[derive(Debug)]
enum ExtraTest {
    Note {
        step: usize,
        note: &'static str,
    },
    Build {
        args: Vec<&'static str>,

        // if set, it checks the modified time of the incremental compilation artifacts before and after the run,
        // and asserts that the timestamp doesn't change
        check_incremental_compilation: Option<CheckIncrementalCompilation>,
    },
    RunBytecode {
        // It should start with "run", "test" or "interpret".
        args: Vec<&'static str>,

        // "run" / "test"
        key: &'static str,
    },
    RunFromSource {
        // It should start with "run" or "test".
        args: Vec<&'static str>,
        key: &'static str,
        check_incremental_compilation: Option<CheckIncrementalCompilation>,
    },
    Clean,  // runs `sodigy clean`

    AssertEq {
        // compares 2 files
        a: &'static str,
        b: &'static str,

        note: &'static str,
    },
    AssertEqRunResults,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CheckIncrementalCompilation {
    All,
    BeforeOptimization,
}

impl CnrContext {
    pub fn extra_tests(&mut self, result: &mut CompileAndRun) {
        if result.error.is_some() || !(
            result.status == Status::CompilePass ||
            result.status == Status::RunTimeout ||
            result.status == Status::RunFail ||
            result.status == Status::RunPass
        ) {
            return;
        }

        let instructions = vec![
            ExtraTest::Note {
                step: 0,
                note: "It runs every possible combination of `sodigy build`, which builds from the code files in `src/`.",
            },
            ExtraTest::Build {
                args: vec!["--emit=bytecode", "-o=bc-debug-0"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--emit=bytecode", "-o=bc-release-0", "--release"],
                check_incremental_compilation: Some(CheckIncrementalCompilation::BeforeOptimization),
            },
            ExtraTest::Build {
                args: vec!["--emit=bytecode-exe", "-o=bcx-debug-0"],
                check_incremental_compilation: Some(CheckIncrementalCompilation::All),
            },
            ExtraTest::Clean,
            ExtraTest::Build {
                args: vec!["--emit=bytecode-exe", "-o=bcx-release-0", "--release"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--emit=c", "-o=c-debug-0"],
                check_incremental_compilation: Some(CheckIncrementalCompilation::BeforeOptimization),
            },
            ExtraTest::Build {
                args: vec!["--emit=c", "-o=c-release-0", "--release"],
                check_incremental_compilation: Some(CheckIncrementalCompilation::All),
            },
            ExtraTest::Clean,
            ExtraTest::Build {
                args: vec!["--emit=exe", "-o=exe-debug-0"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--emit=exe", "-o=exe-release-0", "--release"],
                check_incremental_compilation: None,
            },

            ExtraTest::Note {
                step: 1,
                note: "It runs a few extra `sodigy build` and makes sure that the outputs are deterministic.",
            },
            ExtraTest::Clean,
            ExtraTest::Build {
                args: vec!["--emit=bytecode", "-o=bc-debug-1"],
                check_incremental_compilation: None,
            },
            ExtraTest::Clean,
            ExtraTest::Build {
                args: vec!["--emit=bytecode", "-o=bc-release-1", "--release"],
                check_incremental_compilation: None,
            },
            ExtraTest::Clean,
            ExtraTest::Build {
                args: vec!["--emit=c", "-o=c-debug-1"],
                check_incremental_compilation: None,
            },
            ExtraTest::Clean,
            ExtraTest::Build {
                args: vec!["--emit=c", "-o=c-release-1", "--release"],
                check_incremental_compilation: None,
            },
            ExtraTest::AssertEq {
                a: "bc-debug-0",
                b: "bc-debug-1",
                note: "`sodigy build --emit=bytecode` is not deterministic",
            },
            ExtraTest::AssertEq {
                a: "bc-release-0",
                b: "bc-release-1",
                note: "`sodigy build --emit=bytecode --release` is not deterministic",
            },
            ExtraTest::AssertEq {
                a: "c-debug-0",
                b: "c-debug-1",
                note: "`sodigy build --emit=c` is not deterministic",
            },
            ExtraTest::AssertEq {
                a: "c-release-0",
                b: "c-release-1",
                note: "`sodigy build --emit=c --release` is not deterministic",
            },

            ExtraTest::Note {
                step: 2,
                note: "It runs every possible combination of `sodigy build --bytecode=<path>`.",
            },
            ExtraTest::Build {
                args: vec!["--bytecode=bc-debug-0", "--emit=bytecode-exe", "-o=bcx-debug-1"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--bytecode=bc-debug-0", "--emit=c", "-o=c-debug-2"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--bytecode=bc-debug-0", "--emit=exe", "-o=exe-debug-1"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--bytecode=bc-release-0", "--emit=bytecode-exe", "-o=bcx-release-1"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--bytecode=bc-release-0", "--emit=c", "-o=c-release-2"],
                check_incremental_compilation: None,
            },
            ExtraTest::Build {
                args: vec!["--bytecode=bc-release-0", "--emit=exe", "-o=exe-release-1"],
                check_incremental_compilation: None,
            },

            ExtraTest::Note {
                step: 3,
                // TODO: (run, test) * (bc-debug-0, bc-release-0) * (interpret, native) -> total 8 cases
                note: "It runs `sodigy run --bytecode=<path>` and `sodigy test --bytecode=<path>` with bytecode files from the previous steps, and collects their outputs.",
            },

            ExtraTest::RunBytecode {
                args: vec!["run", "--bytecode=bc-debug-0", "--backend=interpret"],
                key: "run",
            },
            ExtraTest::RunBytecode {
                args: vec!["run", "--bytecode=bc-debug-0", "--backend=native"],
                key: "run",
            },
            ExtraTest::RunBytecode {
                args: vec!["run", "--bytecode=bc-release-0", "--backend=interpret"],
                key: "run",
            },
            ExtraTest::RunBytecode {
                args: vec!["run", "--bytecode=bc-release-0", "--backend=native"],
                key: "run",
            },
            ExtraTest::RunBytecode {
                args: vec!["test", "--bytecode=bc-debug-0", "--backend=interpret"],
                key: "test",
            },
            ExtraTest::RunBytecode {
                args: vec!["test", "--bytecode=bc-debug-0", "--backend=native"],
                key: "test",
            },
            ExtraTest::RunBytecode {
                args: vec!["test", "--bytecode=bc-release-0", "--backend=interpret"],
                key: "test",
            },
            ExtraTest::RunBytecode {
                args: vec!["test", "--bytecode=bc-release-0", "--backend=native"],
                key: "test",
            },

            ExtraTest::Note {
                step: 4,
                note: "It runs C, bytecode-exe and exe files from the previous steps. It uses the system's C compiler to compile the C files. It collects all the outputs.",
            },
            ExtraTest::RunBytecode {
                args: vec!["interpret", "bcx-debug-0"],
                key: "run",
            },
            ExtraTest::RunBytecode {
                args: vec!["interpret", "bcx-debug-0", "--test"],
                key: "test",
            },
            ExtraTest::RunBytecode {
                args: vec!["interpret", "bcx-release-0"],
                key: "run",
            },
            ExtraTest::RunBytecode {
                args: vec!["interpret", "bcx-release-0", "--test"],
                key: "test",
            },

            // TODO: c-debug-0, c-release-0, exe-debug-0, exe-release-0
            //       But how do I set profile for these?

            ExtraTest::AssertEqRunResults,

            ExtraTest::Note {
                step: 5,
                note: "It runs remaining possible combinations of `sodigy run` and `sodigy test` and collects their outputs.",
            },
            ExtraTest::RunFromSource {
                args: vec!["run", "--backend=mir-interpret"],
                key: "run",
            },
            ExtraTest::RunFromSource {
                args: vec!["run", "--backend=interpret"],
                key: "run",
            },
            ExtraTest::RunFromSource {
                args: vec!["run", "--backend=native"],
                key: "run",
            },
            ExtraTest::RunFromSource {
                args: vec!["run", "--backend=mir-interpret", "--release"],
                key: "run",
            },
            ExtraTest::RunFromSource {
                args: vec!["run", "--backend=interpret", "--release"],
                key: "run",
            },
            ExtraTest::RunFromSource {
                args: vec!["run", "--backend=native", "--release"],
                key: "run",
            },
            ExtraTest::RunFromSource {
                args: vec!["test", "--backend=mir-interpret"],
                key: "test",
            },
            ExtraTest::RunFromSource {
                args: vec!["test", "--backend=interpret"],
                key: "test",
            },
            ExtraTest::RunFromSource {
                args: vec!["test", "--backend=native"],
                key: "test",
            },
            ExtraTest::RunFromSource {
                args: vec!["test", "--backend=mir-interpret", "--release"],
                key: "test",
            },
            ExtraTest::RunFromSource {
                args: vec!["test", "--backend=interpret", "--release"],
                key: "test",
            },
            ExtraTest::RunFromSource {
                args: vec!["test", "--backend=native", "--release"],
                key: "test",
            },
            ExtraTest::AssertEqRunResults,

            ExtraTest::Note {
                step: 6,
                note: _,
            },
            ExtraTest::BreakIfLessThan3Files,

            // TODO: if there are more than 2 files, test incremental compilation more thoroughly
        ];

        let mut curr_step = 0;
        let mut curr_note = String::new();

        for instruction in instructions.into_iter() {
            let instruction_str = format!("{instruction:?}");

            match instruction {
                ExtraTest::Note { step, note } => {
                    curr_step = step;
                    curr_note = note.to_string();
                },
                ExtraTest::Build { mut args, check_incremental_compilation } => {
                    let modified_time = match check_incremental_compilation {
                        Some(CheckIncrementalCompilation::All) => todo!(),
                        Some(CheckIncrementalCompilation::BeforeOptimization) => todo!(),
                        None => None,
                    };
                    let args = [vec!["build"], args].concat();

                    if let Err(e) = subprocess::run(
                        &self.sodigy_path,
                        &args,
                        &self.project_dir,
                        30.0,
                        false,  // dump_output
                        true,   // check_nonzero_status
                    ) {
                        result.error = Some(format!("Extra cnr test failure\nstep: {curr_step}\nnote: {curr_note}\ninstruction: {instruction_str}\nerror: {e:?}\nFailed to build a sodigy project!"));
                        return;
                    }
                },
                ExtraTest::RunBytecode { args, key } => {
                    match subprocess::run(
                        &self.sodigy_path,
                        &args,
                        &self.project_dir,
                        30.0,
                        false,  // dump_output
                        false,  // check_nonzero_status
                    ) {
                        Ok(o) => todo!(),
                        Err(e) => {
                            result.error = Some(format!("Extra cnr test failure\nstep: {curr_step}\nnote: {curr_note}\ninstruction: {instruction_str}\nerror: {e:?}\nFailed to run bytecode!"));
                            return;
                        },
                    }
                },
                ExtraTest::Clean => todo!(),
                ExtraTest::AssertEq { a, b, note } => {
                    let path_a = join(&self.project_dir, a).unwrap();
                    let path_b = join(&self.project_dir, b).unwrap();
                    let (content_a, content_b) = match (read_bytes(&path_a), read_bytes(&path_b)) {
                        (Ok(a), Ok(b)) => (String::from_utf8_lossy(&a).to_string(), String::from_utf8_lossy(&b).to_string()),
                        (Err(e), _) => {
                            result.error = Some(format!("Extra cnr test failure\nstep: {curr_step}\nnote: {curr_note}\ninstruction: {instruction_str}\nerror: {e:?}\nFailed to read an asserted file!"));
                            return;
                        },
                        (_, Err(e)) => {
                            result.error = Some(format!("Extra cnr test failure\nstep: {curr_step}\nnote: {curr_note}\ninstruction: {instruction_str}\nerror: {e:?}\nFailed to read an asserted file!"));
                            return;
                        },
                    };

                    if content_a != content_b {
                        todo!();
                    }
                },
            }
        }
    }
}
