use super::{CnrContext, CompileAndRun, Status};

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
    Run {
        // It should start with "run", "test" or "interpret".
        args: Vec<&'static str>,
    },
    Clean,  // runs `sodigy clean`

    AssertEq {
        // compares 2 files
        a: &'static str,
        b: &'static str,

        note: &'static str,
    },
}

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

        let instructions = [
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

            // TODO

            ExtraTest::Note {
                step: 4,
                // bcx-debug-0, bcx-release-0, c-debug-0, c-release-0, exe-debug-0, exe-release-0 -> total 6 cases
                note: "It runs C, bytecode-exe and exe files from the previous steps. It uses the system's C compiler to compile the C files. It collects all the outputs.",
            },

            // TODO

            ExtraTest::Note {
                step: 5,
                note: "It asserts that stdout, stderr and status-code of ALL the runs (whether C, bytecode, exe, ...) from the previous steps are identical.",
            },

            // TODO

            // TODO: `sodigy run`, `sodigy test` from `src/`. Collect their outputs. Also test incremental compilations.

            // TODO: if there are more than 2 files, test incremental compilation more thoroughly
        ];

        todo!()
    }
}
