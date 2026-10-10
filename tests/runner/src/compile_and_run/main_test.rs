use super::{
    CnrContext,
    CompileAndRun,
    LineMatcher,
    Status,
    StatusKind,
    hash_dir,
    match_lines,
    remove_ansi_characters,
};
use crate::subprocess::{self, SubprocessError};
use lazy_static::lazy_static;
use regex::Regex;
use sodigy_fs_api::{FileError, WriteMode, join, join3, read_string, write_string};
use std::time::Instant;

pub struct ExpectedOutput {
    pub compile_stdout: Option<Vec<LineMatcher>>,
    pub compile_stderr: Option<Vec<LineMatcher>>,
    pub run_stdout: Option<Vec<LineMatcher>>,
    pub run_stderr: Option<Vec<LineMatcher>>,
    pub test_stdout: Option<Vec<LineMatcher>>,
    pub test_stderr: Option<Vec<LineMatcher>>,
}

// If it's `{ compile: true, run: true, test: false }`, that means
// `sodigy build` must succeed, `sodigy run` must succeed and `sodigy test` must fail.
// If `.compile` is false, it ignores the other fields.
#[derive(Clone, Debug)]
pub struct ExpectedStatus {
    pub compile: bool,
    pub run: bool,
    pub test: bool,
}

impl ExpectedStatus {
    pub fn all_pass() -> ExpectedStatus {
        ExpectedStatus {
            compile: true,
            run: true,
            test: true,
        }
    }

    pub fn compile_fail() -> ExpectedStatus {
        ExpectedStatus {
            compile: false,

            // Don't care
            run: true,
            test: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Directive {
    pub expected_status: ExpectedStatus,
    pub compile_error: Option<(Comparison, usize)>,
    pub compile_warning: Option<(Comparison, usize)>,
    pub test_error: Option<(Comparison, usize)>,
}

impl CnrContext {
    // Build and run the test case, and compare the output with the expected output.
    pub fn main_test(&self) -> CompileAndRun {
        let lib_src = join3(&self.project_dir, "src", "lib.sdg").unwrap();
        let directive = parse_directive(&lib_src).unwrap();
        let build_stdout_colored: Option<Vec<u8>>;
        let build_stderr_colored: Option<Vec<u8>>;
        let mut run_stdout_colored: Option<Vec<u8>> = None;
        let mut run_stderr_colored: Option<Vec<u8>> = None;
        let mut test_stdout_colored: Option<Vec<u8>> = None;
        let mut test_stderr_colored: Option<Vec<u8>> = None;
        let mut status = Status {
            prepare: StatusKind::NotRunYet,
            build: StatusKind::NotRunYet,
            test: StatusKind::NotRunYet,
            run: StatusKind::NotRunYet,
        };

        // TODO: do we have to hash expected_output?
        let hash = format!("{:024x}", hash_dir(&join(&self.project_dir, "src").unwrap()));

        if !directive.expected_status.compile && self.expected_output.compile_stderr.is_none() {
            panic!("If you want to assert that `{}` fails to compile, please add `{}.compile.stderr` file.", self.name, self.name);
        }

        if let (Ok(o1), Ok(o2)) = (
            subprocess::run(&self.sodigy_path, &["check"], &self.project_dir, 30.0, false, false),
            subprocess::run(&self.sodigy_path, &["check", "--test"], &self.project_dir, 30.0, false, false),
        ) {
            // It's a cnr case that has no other compile error, but doesn't have an entry.
            // The harness will insert an entry. Otherwise, it'd be too annoying to create a
            // cnr case if all case have to have an entry.
            if let (Some(11), Some(0)) = (o1.code(), o2.code()) {
                let lib = join3(&self.project_dir, "src", "lib.sdg").unwrap();
                write_string(
                    &lib,
                    "\n\n#[entry] fn main_entry_1234_i_hope_there_s_no_name_collision() = 0;",
                    WriteMode::AlwaysAppend,
                ).unwrap();
            }
        }

        match subprocess::run(&self.sodigy_path, &["clean"], &self.project_dir, 5.0, false, false) {
            Ok(output) if !output.success() => {
                status.prepare = StatusKind::Fail;
                return CompileAndRun {
                    name: self.name.to_string(),
                    error: Some(format!("error with `sodigy clean` (exit status {:?})", output.code())),
                    status,
                    hash,
                    ..CompileAndRun::default()
                };
            },
            Err(e) => {
                status.prepare = StatusKind::Fail;
                return CompileAndRun {
                    name: self.name.to_string(),
                    error: Some(format!("error with `sodigy clean`: {e:?}")),
                    status,
                    hash,
                    ..CompileAndRun::default()
                };
            },
            _ => {},
        }

        // TODO: collect timings data... for all cnrs!
        let mut args_run = vec!["build", "--emit=bytecode-exe", "--dump-timings"];

        if self.dump_post_mir_log {
            args_run.push("--dump-post-mir-log");
        }

        // The cnr test runner has to validate the spans of the tokens. But it doesn't
        // have to validate the tokens in std, because they're the same!
        // Ideally, we have to run `--validate-token-spans` only when `self.cnr_seq == 0`,
        // but there might be a lexer error in the first cnr, no one knows. So we run
        // `--validate-token-spans` for the first 5 cases.
        match self.cnr_seq {
            ..5 => {
                args_run.push("--validate-token-spans");
            },
            _ => {
                args_run.push("--validate-lib-token-spans");
            },
        }

        let mut args_test = args_run.clone();
        args_test.push("--test");
        args_test.push("-o=target/test");
        args_run.push("-o=target/run");

        let build_started_at = Instant::now();
        let output = match subprocess::run(
             &self.sodigy_path,
            &args_run,
            &self.project_dir,
            30.0,  // timeout (s)
            self.dump_output,
            false,  // check_nonzero_status
        ) {
            Ok(output) => output,
            Err(e) => {
                let error = match e {
                    SubprocessError::Timeout => {
                        status.build = StatusKind::Timeout;
                        String::from("build-timeout")
                    },
                    e => {
                        status.build = StatusKind::Fail;
                        format!("error with `sodigy build -o=target/run: {e:?}`")
                    },
                };

                return CompileAndRun {
                    name: self.name.to_string(),
                    error: Some(error),
                    status,
                    build_elapsed_ms: Instant::now().duration_since(build_started_at).as_millis() as u64,
                    hash,
                    ..CompileAndRun::default()
                };
            },
        };

        if self.debug_bytecode {
            std::process::Command::new(&self.sodigy_path)
                // TODO: choose profile: target/run vs target/test
                .args(&["interpret", "target/run", "--debug-bytecode"])
                .current_dir(&self.project_dir)
                .stdin(std::process::Stdio::inherit())
                .status()
                .unwrap();

            return CompileAndRun::default();
        }

        let build_elapsed_ms = Instant::now().duration_since(build_started_at).as_millis() as u64;
        build_stdout_colored = Some(output.stdout.to_vec());
        build_stderr_colored = Some(output.stderr.to_vec());

        let mut error = match check_build_output(&output, &directive, &self.expected_output) {
            Ok(()) => None,
            Err(e) => Some(e),
        };

        // If `sodigy build -o=target/run` succeeds and `sodigy build -o=target/test --test` fails,
        // then it'd be very tough to debug. Let's hope that never happens.
        if let Err(e) = subprocess::run(
            &self.sodigy_path,
            &args_test,
            &self.project_dir,
            30.0,  // timeout (s)
            self.dump_output,
            false,  // check_nonzero_status
        ) {
            // eprintln vs return Err
            todo!()
        }

        let mut test_elapsed_ms = None;
        let mut run_elapsed_ms = None;
        status.prepare = StatusKind::Pass;
        status.build = output.status.success().into();

        if status.build == StatusKind::Pass {
            let test_started_at = Instant::now();
            match subprocess::run(
                &self.sodigy_path,
                &["interpret", "target/test"],
                &self.project_dir,
                30.0,   // timeout
                self.dump_output,
                false,  // check_nonzero_status
            ) {
                Ok(output) => {
                    test_elapsed_ms = Some(Instant::now().duration_since(test_started_at).as_millis() as u64);
                    test_stdout_colored = Some(output.stdout.to_vec());
                    test_stderr_colored = Some(output.stderr.to_vec());

                    error = match (error, check_test_output(&output, &directive, &self.expected_output)) {
                        (None, Err(e)) => Some(e),
                        (e, _) => e,
                    };

                    status.test = output.status.success().into();
                },
                Err(SubprocessError::Timeout) => {
                    test_elapsed_ms = Some(Instant::now().duration_since(test_started_at).as_millis() as u64);
                    error = Some(String::from("test-timeout"));
                    status.test = StatusKind::Timeout;
                },
                Err(e) => {
                    status.test = StatusKind::Fail;
                    return CompileAndRun {
                        name: self.name.to_string(),
                        error: Some(format!("error with `sodigy interpret target/test`: {e:?}")),
                        status,
                        build_elapsed_ms,
                        hash,
                        ..CompileAndRun::default()
                    };
                },
            }

            let run_started_at = Instant::now();
            match subprocess::run(
                &self.sodigy_path,
                &["interpret", "target/run"],
                &self.project_dir,
                30.0,   // timeout
                self.dump_output,
                false,  // check_nonzero_status
            ) {
                Ok(output) => {
                    run_elapsed_ms = Some(Instant::now().duration_since(run_started_at).as_millis() as u64);
                    run_stdout_colored = Some(output.stdout.to_vec());
                    run_stderr_colored = Some(output.stderr.to_vec());

                    error = match (error, check_run_output(&output, &directive, &self.expected_output)) {
                        (None, Err(e)) => Some(e),
                        (e, _) => e,
                    };

                    status.run = output.status.success().into();
                },
                Err(SubprocessError::Timeout) => {
                    run_elapsed_ms = Some(Instant::now().duration_since(run_started_at).as_millis() as u64);
                    error = Some(String::from("run-timeout"));
                    status.run = StatusKind::Timeout;
                },
                Err(e) => {
                    status.run = StatusKind::Fail;
                    return CompileAndRun {
                        name: self.name.to_string(),
                        error: Some(format!("error with `sodigy interpret target/run`: {e:?}")),
                        status,
                        build_elapsed_ms,
                        hash,
                        ..CompileAndRun::default()
                    };
                },
            }
        }

        let build_stdout_colored = build_stdout_colored.as_ref().map(|s| String::from_utf8_lossy(s).to_string());
        let build_stderr_colored = build_stderr_colored.as_ref().map(|s| String::from_utf8_lossy(s).to_string());
        let run_stdout_colored = run_stdout_colored.as_ref().map(|s| String::from_utf8_lossy(s).to_string());
        let run_stderr_colored = run_stderr_colored.as_ref().map(|s| String::from_utf8_lossy(s).to_string());
        let test_stdout_colored = test_stdout_colored.as_ref().map(|s| String::from_utf8_lossy(s).to_string());
        let test_stderr_colored = test_stderr_colored.as_ref().map(|s| String::from_utf8_lossy(s).to_string());

        CompileAndRun {
            name: self.name.to_string(),
            error,
            build_stdout: remove_ansi_characters(&build_stdout_colored),
            build_stderr: remove_ansi_characters(&build_stderr_colored),
            run_stdout: remove_ansi_characters(&run_stdout_colored),
            run_stderr: remove_ansi_characters(&run_stderr_colored),
            test_stdout: remove_ansi_characters(&test_stdout_colored),
            test_stderr: remove_ansi_characters(&test_stderr_colored),
            status,
            build_stdout_colored,
            build_stderr_colored,
            run_stdout_colored,
            run_stderr_colored,
            test_stdout_colored,
            test_stderr_colored,
            hash,
            build_elapsed_ms,
            run_elapsed_ms,
            test_elapsed_ms,
        }
    }
}

fn parse_directive(file_path: &str) -> Result<Directive, FileError> {
    fn error(file: &str, line: &str) -> ! {
        panic!("Error while parsing directive!\nFile: `{file}`\nLine: `{line}`")
    }

    let s = read_string(&file_path)?;
    let mut expected_compile: Option<bool> = None;
    let mut expected_run: Option<bool> = None;
    let mut expected_test: Option<bool> = None;
    let mut compile_error: Option<(Comparison, usize)> = None;
    let mut compile_warning: Option<(Comparison, usize)> = None;
    let mut test_error: Option<(Comparison, usize)> = None;

    for line in s.lines() {
        if line.starts_with("//%") {
            let directive = line.strip_prefix("//%").unwrap().trim();

            match directive {
                "compile-pass" => match (expected_compile, expected_run, expected_test) {
                    (Some(_), _, _) => error(file_path, line),
                    _ => { expected_compile = Some(true); },
                },
                "compile-fail" => match (expected_compile, expected_run, expected_test) {
                    (Some(_), _, _) | (_, Some(_), _) | (_, _, Some(_)) => error(file_path, line),
                    _ => { expected_compile = Some(false); },
                },
                "run-pass" => match (expected_compile, expected_run, expected_test) {
                    (Some(false), _, _) | (_, Some(_), _) => error(file_path, line),
                    _ => { expected_run = Some(true); },
                },
                "run-fail" => match (expected_compile, expected_run, expected_test) {
                    (Some(false), _, _) | (_, Some(_), _) => error(file_path, line),
                    _ => { expected_run = Some(false); },
                },
                "test-pass" => match (expected_compile, expected_run, expected_test) {
                    (Some(false), _, _) | (_, _, Some(_)) => error(file_path, line),
                    _ => { expected_test = Some(true); },
                },
                "test-fail" => match (expected_compile, expected_run, expected_test) {
                    (Some(false), _, _) | (_, _, Some(_)) => error(file_path, line),
                    _ => { expected_test = Some(false); },
                },
                _ if directive.starts_with("compile-error") || directive.starts_with("compile-warning") || directive.starts_with("test-error") => {
                    let (kind, directive) = match directive {
                        _ if directive.starts_with("compile-error") => ("ce", directive.get(13..).unwrap().trim()),
                        _ if directive.starts_with("compile-warning") => ("cw", directive.get(15..).unwrap().trim()),
                        _ if directive.starts_with("test-error") => ("te", directive.get(10..).unwrap().trim()),
                        _ => error(file_path, line),
                    };
                    let (cmp, directive) = match directive {
                        _ if directive.starts_with(">=") => (Comparison::Geq, directive.get(2..).unwrap().trim()),
                        _ if directive.starts_with(">") => (Comparison::Gt, directive.get(1..).unwrap().trim()),
                        _ if directive.starts_with("<=") => (Comparison::Leq, directive.get(2..).unwrap().trim()),
                        _ if directive.starts_with("<") => (Comparison::Lt, directive.get(1..).unwrap().trim()),
                        _ if directive.starts_with("!=") => (Comparison::Neq, directive.get(2..).unwrap().trim()),
                        _ if directive.starts_with("==") => (Comparison::Eq, directive.get(2..).unwrap().trim()),
                        _ => error(file_path, line),
                    };
                    let n = match directive.parse::<usize>() {
                        Ok(n) => n,
                        Err(_) => error(file_path, line),
                    };

                    match kind {
                        "ce" => match compile_error {
                            Some(_) => error(file_path, line),
                            None => { compile_error = Some((cmp, n)); },
                        },
                        "cw" => match compile_warning {
                            Some(_) => error(file_path, line),
                            None => { compile_warning = Some((cmp, n)); },
                        },
                        "te" => match test_error {
                            Some(_) => error(file_path, line),
                            None => { test_error = Some((cmp, n)); },
                        },
                        _ => unreachable!(),
                    }
                },
                _ => error(file_path, line),
            }
        }
    }

    let mut expected_status = match (expected_compile, expected_run, expected_test) {
        (Some(false), _, _) => Some(ExpectedStatus::compile_fail()),
        (None, None, None) => None,
        (_, run, test) => Some(ExpectedStatus {
            compile: true,
            run: run.unwrap_or(true),
            test: test.unwrap_or(true),
        }),
    };

    // If the user expects compile errors, that implies compile-fail!
    if let Some((cmp, n)) = compile_error && expected_status.is_none() {
        match (cmp, n) {
            (Comparison::Gt | Comparison::Geq, _) => {
                expected_status = Some(ExpectedStatus::compile_fail());
            },
            (Comparison::Eq, n) if n != 0 => {
                expected_status = Some(ExpectedStatus::compile_fail());
            },
            _ => {},
        }
    }

    if let Some((cmp, n)) = test_error && expected_status.is_none() {
        match (cmp, n) {
            (Comparison::Gt | Comparison::Geq, _) => {
                expected_status = Some(ExpectedStatus {
                    compile: true,
                    run: true,
                    test: false,
                });
            },
            (Comparison::Eq, n) if n != 0 => {
                expected_status = Some(ExpectedStatus {
                    compile: true,
                    run: true,
                    test: false,
                });
            },
            _ => {},
        }
    }

    Ok(Directive {
        expected_status: expected_status.unwrap_or(ExpectedStatus::all_pass()),
        compile_error,
        compile_warning,
        test_error,
    })
}

fn check_build_output(output: &subprocess::Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    let (compile_errors, compile_warnings) = match count_compile_errors_and_warnings(&String::from_utf8_lossy(&output.stderr)) {
        Some((e, w)) => (e, w),
        None => {
            return Err(String::from("failed to parse the compiler output"));
        },
    };

    // TODO: The term `build` and `compile` are mixed and is confusing.
    //       The user-facing API of the test harness uses the term `compile`,
    //       but everyone else uses the term `build`.
    match (output.status.success(), directive.expected_status.compile) {
        (true, false) => { return Err(String::from("expected compile-fail, but it passed")); },
        (false, true) => { return Err(String::from("expected compile-pass, but it failed")); },
        _ => {},
    }

    if let Some((cmp, n)) = directive.compile_error {
        cmp.check(n, compile_errors, "the number of compile errors")?;
    }

    if let Some((cmp, n)) = directive.compile_warning {
        cmp.check(n, compile_warnings, "the number of compile warnings")?;
    }

    match_lines(&String::from_utf8_lossy(&output.stdout), &expected_output.compile_stdout).map_err(|e| format!("expected compile.stdout and actual stdout do not match\n{e}"))?;
    match_lines(&String::from_utf8_lossy(&output.stderr), &expected_output.compile_stderr).map_err(|e| format!("expected compile.stderr and actual stderr do not match\n{e}"))?;
    Ok(())
}

fn check_run_output(output: &subprocess::Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    match (output.status.success(), directive.expected_status.run) {
        (true, false) => { return Err(String::from("expected run-fail, but it passed")); },
        (false, true) => { return Err(String::from("expected run-pass, but it failed")); },
        _ => {},
    }

    match_lines(&String::from_utf8_lossy(&output.stdout), &expected_output.run_stdout).map_err(|e| format!("expected run.stdout and actual stdout do not match\n{e}"))?;
    match_lines(&String::from_utf8_lossy(&output.stderr), &expected_output.run_stderr).map_err(|e| format!("expected run.stderr and actual stderr do not match\n{e}"))?;
    Ok(())
}

fn check_test_output(output: &subprocess::Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    match (output.status.success(), directive.expected_status.test) {
        (true, false) => { return Err(String::from("expected test-fail, but it passed")); },
        (false, true) => { return Err(String::from("expected test-pass, but it failed")); },
        _ => {},
    }

    if let Some((cmp, n)) = directive.test_error {
        todo!();
    }

    match_lines(&String::from_utf8_lossy(&output.stdout), &expected_output.test_stdout).map_err(|e| format!("expected test.stdout and actual stdout do not match\n{e}"))?;
    match_lines(&String::from_utf8_lossy(&output.stderr), &expected_output.test_stderr).map_err(|e| format!("expected test.stderr and actual stderr do not match\n{e}"))?;
    Ok(())
}

lazy_static! {
    static ref COMPILER_RESULT_RE: Regex = Regex::new(r"^Finished\:\s(\d+)\serror(?:s)?\sand\s(\d+)\swarning(?:s)?.+").unwrap();
}

fn count_compile_errors_and_warnings(output: &str) -> Option<(usize, usize)> {
    for line in output.lines().rev() {
        if let Some(c) = COMPILER_RESULT_RE.captures(line) {
            return Some((
                c.get(1).unwrap().as_str().parse::<usize>().unwrap(),
                c.get(2).unwrap().as_str().parse::<usize>().unwrap(),
            ));
        }
    }

    None
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Comparison {
    Gt,
    Geq,
    Lt,
    Leq,
    Eq,
    Neq,
}

impl Comparison {
    pub fn check(&self, lhs: usize, rhs: usize, key: &str) -> Result<(), String> {
        match self {
            Comparison::Gt if lhs > rhs => Ok(()),
            Comparison::Gt => Err(format!("expected {key} to be greater than {lhs}, but got {rhs}")),
            Comparison::Geq if lhs >= rhs => Ok(()),
            Comparison::Geq => Err(format!("expected {key} to be greater than or equal to {lhs}, but got {rhs}")),
            Comparison::Lt if lhs < rhs => Ok(()),
            Comparison::Lt => Err(format!("expected {key} to be less than {lhs}, but got {rhs}")),
            Comparison::Leq if lhs <= rhs => Ok(()),
            Comparison::Leq => Err(format!("expected {key} to be less than or equal to {lhs}, but got {rhs}")),
            Comparison::Eq if lhs == rhs => Ok(()),
            Comparison::Eq => Err(format!("expected {key} to be {lhs}, but got {rhs}")),
            Comparison::Neq if lhs != rhs => Ok(()),
            Comparison::Neq => Err(format!("expected {key} not to be {lhs}, but is {rhs}")),
        }
    }
}

