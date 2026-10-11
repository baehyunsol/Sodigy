use super::{CnrContext, CompileAndRun};
use sodigy_code_gen::Emit;
use sodigy_driver::Backend;
use sodigy_subprocess::{self as subprocess, Output, SubprocessError};
use sodigy_fs_api::read_bytes;

// `sodigy build` can take 3 different kinds of input.
#[derive(Clone, Copy, Debug)]
enum Input {
    Source,
    ReadableBytecode,
    ExecutableBytecode,
}

// A `sodigy build` command.
#[derive(Clone, Debug)]
struct Build {
    input: Input,
    optimize: bool,
    test_mode: bool,
    output: Emit,
}

impl Build {
    pub fn is_deterministic(&self) -> bool {
        match self.output {
            // I don't trust the rust compiler.
            Emit::Exe => false,
            _ => true,
        }
    }

    pub fn output_path(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            match self.input {
                Input::Source => "sdg",
                Input::ReadableBytecode => "bytecode",
                Input::ExecutableBytecode => "bytecode-exe",
            },
            if self.optimize { "o" } else { "d" },
            if self.test_mode { "t" } else { "r" },
            self.output.flag(),
        )
    }

    pub fn command(&self, std_assertions: bool) -> String {
        format!(
            "sodigy build{}{}{}{} --emit={} -o={}",
            match self.input {
                Input::Source => String::new(),
                Input::ReadableBytecode => {
                    let bytecode_artifact = Build {
                        input: Input::Source,
                        output: Emit::ReadableBytecode,
                        ..self.clone()
                    };
                    format!(" --bytecode={}", bytecode_artifact.output_path())
                },
                Input::ExecutableBytecode => {
                    let bytecode_artifact = Build {
                        input: Input::Source,
                        output: Emit::ExecutableBytecode,
                        ..self.clone()
                    };
                    format!(" --bytecode={}", bytecode_artifact.output_path())
                },
            },
            if self.optimize { " --release" } else { "" },
            if self.test_mode { " --test" } else { "" },
            if std_assertions { " --std-assertions" } else { "" },
            self.output.flag(),
            self.output_path(),
        )
    }

    pub fn build(
        &self,
        sodigy_path: &str,
        project_dir: &str,
        std_assertions: bool,
    ) -> Result<(), SubprocessError> {
        let command = self.command(std_assertions);
        let command: Vec<&str> = command.split_whitespace().collect();

        subprocess::run(
            sodigy_path,
            &command[1..],
            project_dir,
            30.0,
            false,
            true,
        ).map(|_| ())
    }

    pub fn run(&self) -> Result<Output, SubprocessError> {
        let output = self.output_path();

        match self.output {
            Emit::ReadableBytecode | Emit::ExecutableBytecode => todo!(),  // use `sodigy interpret` to run
            Emit::Rust => todo!(),  // use `rustc` to run
            Emit::Exe => todo!(),  // just run
            Emit::Nothing => unreachable!(),
        }
    }

    // I'm too lazy to implement an iterator haha
    pub fn all() -> Vec<Build> {
        let mut result = vec![];

        // `input` must be the outer-most loop and `Input::Source` must be the first because,
        // in order for `Input::ReadableBytecode` and `Input::ExecutableBytecode` to work,
        // bytecode must be built!
        for input in [Input::Source, Input::ReadableBytecode, Input::ExecutableBytecode] {
            for optimize in [true, false] {
                for test_mode in [true, false] {
                    for output in Emit::all() {
                        if output == Emit::Nothing {
                            continue;
                        }

                        result.push(Build { input, optimize, test_mode, output });
                    }
                }
            }
        }

        result
    }
}

// A `sodigy run` or `sodigy test` command.
struct Run {
    input: Input,
    optimize: bool,
    test_mode: bool,
    backend: Backend,
}

impl Run {
    pub fn all() -> Vec<Run> {
        todo!()
    }

    pub fn command(&self) -> String {
        todo!()
    }

    pub fn run(&self) -> Result<Output, SubprocessError> {
        todo!()
    }
}

impl CnrContext {
    pub fn extra_tests(&mut self, result: &mut CompileAndRun) {
        if result.error.is_some() || result.status.failed_to_build() {
            return;
        }

        // loop 1. Build the project with different options.
        for (i, build) in Build::all().into_iter().enumerate() {
            // It regularly runs `sodigy clean` to stress-test the incremental compilation.
            if i % 3 == 0 {
                if let Err(e) = clean(&self.sodigy_path, &self.project_dir) {
                    result.error = Some(format!("error at extra_tests\nloop: 1\niter: {i}\ncommand: sodigy clean\nerror: {e:?}"));
                    return;
                }
            }

            if let Err(e) = build.build(&self.sodigy_path, &self.project_dir, self.std_assertions) {
                result.error = Some(format!("error at extra_tests\nloop: 1\niter: {i}\ncommand: {}\nerror: {e:?}", build.command(self.std_assertions)));
                return;
            }
        }

        // loop 2. Build the project with different options, again.
        //         Some options are supposed to be deterministic (e.g. bytecode output).
        //         It checks if the previous output and the current output are identical.
        for (i, build) in Build::all().into_iter().enumerate() {
            if !build.is_deterministic() {
                continue;
            }

            // It regularly runs `sodigy clean` to stress-test the incremental compilation.
            if i % 3 == 0 {
                if let Err(e) = clean(&self.sodigy_path, &self.project_dir) {
                    result.error = Some(format!("error at extra_tests\nloop: 2\niter: {i}\ncommand: sodigy clean\nerror: {e:?}"));
                    return;
                }
            }

            let prev_result = read_bytes(&build.output_path()).unwrap();

            if let Err(e) = build.build(&self.sodigy_path, &self.project_dir, self.std_assertions) {
                result.error = Some(format!("error at extra_tests\nloop: 2\niter: {i}\ncommand: {}\nerror: {e:?}", build.command(self.std_assertions)));
                return;
            }

            let curr_result = read_bytes(&build.output_path()).unwrap();

            if prev_result != curr_result {
                result.error = Some(format!("error at extra_tests\nloop: 2\niter: {i}\ncommand: {}\ndiff: (TODO)", build.command(self.std_assertions)));
                return;
            }
        }

        let mut run_output: Option<Output> = None;
        let mut test_output: Option<Output> = None;

        // loop 3. Run the built outputs. Outputs with the same profiles
        //         must have the same behavior (stdout, stderr, exit-code).
        for (i, build) in Build::all().into_iter().enumerate() {
            let output = match build.run() {
                Ok(o) => o,
                Err(e) => {
                    result.error = Some(format!("error at extra_tests\nloop: 3\niter: {i}\ncommand: {}\nerror: {e:?}", build.command(self.std_assertions)));
                    return;
                },
            };

            match (build.test_mode, &run_output, &test_output) {
                (true, _, None) => { test_output = Some(output); },
                (true, _, Some(o)) => {
                    if o != &output {
                        result.error = Some(format!("error at extra_tests\nloop: 3\niter: {i}\ncommand: {}\ndiff: (TODO)", build.command(self.std_assertions)));
                        return;
                    }
                },
                (false, None, _) => { run_output = Some(output); },
                (false, Some(o), _) => {
                    if o != &output {
                        result.error = Some(format!("error at extra_tests\nloop: 3\niter: {i}\ncommand: {}\ndiff: (TODO)", build.command(self.std_assertions)));
                        return;
                    }
                },
            }
        }

        let run_output = run_output.unwrap();
        let test_output = test_output.unwrap();

        // loop 4. Run the project with `sodigy run` or `sodigy test`.
        //         Their output (stdout, stderr, exit-code) must be the
        //         same with those in loop 3.
        for (i, run) in Run::all().into_iter().enumerate() {
            // It regularly runs `sodigy clean` to stress-test the incremental compilation.
            if i % 3 == 0 {
                if let Err(e) = clean(&self.sodigy_path, &self.project_dir) {
                    result.error = Some(format!("error at extra_tests\nloop: 4\niter: {i}\ncommand: sodigy clean\nerror: {e:?}"));
                    return;
                }
            }

            let output = match run.run() {
                Ok(o) => o,
                Err(e) => {
                    result.error = Some(format!("error at extra_tests\nloop: 4\niter: {i}\ncommand: {}\nerror: {e:?}", run.command()));
                    return;
                },
            };

            if run.test_mode && test_output != output {
                result.error = Some(format!("error at extra_tests\nloop: 4\niter: {i}\ncommand: {}\ndiff: (TODO)", run.command()));
                return;
            } else if !run.test_mode && run_output != output {
                result.error = Some(format!("error at extra_tests\nloop: 4\niter: {i}\ncommand: {}\ndiff: (TODO)", run.command()));
                return;
            }
        }

        if self.sdg_files < 3 {
            return;
        }

        // TODO: more thorough incremental compilation test
        // 1. sodigy clean
        // 2. remove foo.sdg
        // 3. sodigy run --backend=native
        // 4. restore foo.sdg
        // 5. sodigy run --backend=native --release
        // 6. add erroneous statements to bar.sdg
        // 7. sodigy run --backend=native
        // 8. sodigy run --backend=native
        // 9. restore bar.sdg
        // 10. sodigy run --backend=native --release
        // 11. sodigy run --backend=native
        // 12. add a tmp module (and also define it in lib.sdg)
        // 13. sodigy test --backend=native
        // 14. sodigy test --backend=native --release
        // 15. add harmless statements to tmp
        // 16. sodigy test --backend=native --release
        // 17. add harmful statements to tmp (compile-pass, run-fail)
        // 18. sodigy test --backend=native --release
        // 19. sodigy test --backend=interpret
        // 20. remove tmp.sdg and restore lib.sdg
        // 21. sodigy test --backend=interpret --release
        // 22. sodigy test --backend=native
    }
}

fn clean(sodigy_path: &str, project_dir: &str) -> Result<(), SubprocessError> {
    subprocess::run(
        sodigy_path,
        &["clean"],
        project_dir,
        5.0,
        false,
        true,
    ).map(|_| ())
}

