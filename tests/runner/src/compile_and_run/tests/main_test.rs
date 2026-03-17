use super::{
    CnrContext,
    CompileAndRun,
    LineMatcher,
    Status,
    hash_dir,
    match_lines,
    remove_ansi_characters,
};
use crate::subprocess::{self, SubprocessError};
use lazy_static::lazy_static;
use regex::Regex;
use sodigy_fs_api::{FileError, join, join3, read_string};
use std::time::Instant;

pub struct ExpectedOutput {
    pub compile_stdout: Option<Vec<LineMatcher>>,
    pub compile_stderr: Option<Vec<LineMatcher>>,
    pub run_stdout: Option<Vec<LineMatcher>>,
    pub run_stderr: Option<Vec<LineMatcher>>,
}

#[derive(Clone, Debug)]
pub struct Directive {
    pub expected_status: Status,
    pub compile_error: Option<(Comparison, usize)>,
    pub compile_warning: Option<(Comparison, usize)>,
    pub run_error: Option<(Comparison, usize)>,
}

impl CnrContext {
    // Build and run the test case, and compare the output with the expected output.
    pub fn main_test(&self) -> CompileAndRun {
        let lib_src = join3(&self.project_dir, "src", "lib.sdg").unwrap();
        let directive = parse_directive(&lib_src).unwrap();
        let mut stdout_colored = vec![];
        let mut stderr_colored = vec![];

        // TODO: do we have to hash expected_output?
        let hash = format!("{:024x}", hash_dir(&join(&self.project_dir, "src").unwrap()));

        if directive.expected_status == Status::CompileFail && self.expected_output.compile_stderr.is_none() {
            panic!("If you want to assert that `{}` fails to compile, please add `{}.compile.stderr` file.", self.name, self.name);
        }

        // It's not using a tmp project, and there may be compilation artifacts from previous tests.
        if self.sdg_files > 1 {
            match subprocess::run(&self.sodigy_path, &["clean"], &self.project_dir, 5.0, false, false) {
                Ok(output) if !output.success() => {
                    return CompileAndRun {
                        name: self.name.to_string(),
                        error: Some(format!("error with `sodigy clean` (exit status {:?})", output.code())),
                        status: Status::MiscError,
                        hash,
                        ..CompileAndRun::default()
                    };
                },
                Err(e) => {
                    return CompileAndRun {
                        name: self.name.to_string(),
                        error: Some(format!("error with `sodigy clean`: {e:?}")),
                        status: Status::MiscError,
                        hash,
                        ..CompileAndRun::default()
                    };
                },
                _ => {},
            }
        }

        let compile_started_at = Instant::now();
        let output = match subprocess::run(
            &self.sodigy_path,
            &["build", "--test", "-o=target/run", "--emit-irs"],
            &self.project_dir,
            30.0,
            self.dump_output,
            false,
        ) {
            Ok(output) => output,
            Err(e) => {
                let (error, status) = match e {
                    SubprocessError::Timeout => (String::from("compile-timeout"), Status::CompileTimeout),
                    e => (format!("error with `sodigy build --test -o=target/run: {e:?}`"), Status::MiscError),
                };

                return CompileAndRun {
                    name: self.name.to_string(),
                    error: Some(error),
                    status,
                    compile_elapsed_ms: Instant::now().duration_since(compile_started_at).as_millis() as u64,
                    hash,
                    ..CompileAndRun::default()
                };
            },
        };

        let compile_elapsed_ms = Instant::now().duration_since(compile_started_at).as_millis() as u64;
        stdout_colored.extend(&output.stdout);
        stderr_colored.extend(&output.stderr);

        let mut error = match check_compile_output(&output, &directive, &self.expected_output) {
            Ok(()) => None,
            Err(e) => Some(e),
        };
        let mut status = if output.status.success() { Status::CompilePass } else { Status::CompileFail };
        let mut run_elapsed_ms = None;

        if status != Status::CompileFail {
            let run_started_at = Instant::now();
            match subprocess::run(
                &self.sodigy_path,
                &["interpret", "target/run"],
                &self.project_dir,
                30.0,
                self.dump_output,
                false,
            ) {
                Ok(output) => {
                    run_elapsed_ms = Some(Instant::now().duration_since(run_started_at).as_millis() as u64);
                    stdout_colored.extend(&output.stdout);
                    stderr_colored.extend(&output.stderr);

                    error = match (error, check_run_output(&output, &directive, &self.expected_output)) {
                        (None, Err(e)) => Some(e),
                        (e, _) => e,
                    };

                    status = if output.status.success() { Status::RunPass } else { Status::RunFail };
                },
                Err(SubprocessError::Timeout) => {
                    run_elapsed_ms = Some(Instant::now().duration_since(run_started_at).as_millis() as u64);
                    error = Some(String::from("run-timeout"));
                    status = Status::RunTimeout;
                },
                Err(e) => {
                    return CompileAndRun {
                        name: self.name.to_string(),
                        error: Some(format!("error with `sodigy interpret target/run`: {e:?}")),
                        status: Status::MiscError,
                        compile_elapsed_ms,
                        hash,
                        ..CompileAndRun::default()
                    };
                },
            }
        }

        let stdout_colored = String::from_utf8_lossy(&stdout_colored).to_string();
        let stderr_colored = String::from_utf8_lossy(&stderr_colored).to_string();

        CompileAndRun {
            name: self.name.to_string(),
            error,
            stdout: remove_ansi_characters(&stdout_colored),
            stderr: remove_ansi_characters(&stderr_colored),
            status,
            stdout_colored,
            stderr_colored,
            hash,
            compile_elapsed_ms,
            run_elapsed_ms,
        }
    }
}

fn parse_directive(file_path: &str) -> Result<Directive, FileError> {
    fn error(file: &str, line: &str) -> ! {
        panic!("Error while parsing directive!\nFile: `{file}`\nLine: `{line}`")
    }

    let s = read_string(&file_path)?;
    let mut expected_status = None;
    let mut compile_error = None;
    let mut compile_warning = None;
    let mut run_error = None;

    for line in s.lines() {
        if line.starts_with("//%") {
            let directive = line.strip_prefix("//%").unwrap().trim();

            match directive {
                "compile-pass" => match expected_status {
                    Some(_) => error(file_path, line),
                    None => { expected_status = Some(Status::CompilePass); },
                },
                "compile-fail" => match expected_status {
                    Some(_) => error(file_path, line),
                    None => { expected_status = Some(Status::CompileFail); },
                },
                "run-pass" => match expected_status {
                    Some(_) => error(file_path, line),
                    None => { expected_status = Some(Status::RunPass); },
                },
                "run-fail" => match expected_status {
                    Some(_) => error(file_path, line),
                    None => { expected_status = Some(Status::RunFail); },
                },
                _ if directive.starts_with("compile-error") || directive.starts_with("compile-warning") || directive.starts_with("run-error") => {
                    let (kind, directive) = match directive {
                        _ if directive.starts_with("compile-error") => ("ce", directive.get(13..).unwrap().trim()),
                        _ if directive.starts_with("compile-warning") => ("cw", directive.get(15..).unwrap().trim()),
                        _ if directive.starts_with("run-error") => ("re", directive.get(9..).unwrap().trim()),
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
                        "re" => match run_error {
                            Some(_) => error(file_path, line),
                            None => { run_error = Some((cmp, n)); },
                        },
                        _ => unreachable!(),
                    }
                },
                _ => error(file_path, line),
            }
        }
    }

    // If the user expects compile errors, that implies compile-fail!
    if let Some((cmp, n)) = compile_error && expected_status.is_none() {
        match (cmp, n) {
            (Comparison::Gt | Comparison::Geq, _) => {
                expected_status = Some(Status::CompileFail);
            },
            (Comparison::Eq, n) if n != 0 => {
                expected_status = Some(Status::CompileFail);
            },
            _ => {},
        }
    }

    if let Some((cmp, n)) = run_error && expected_status.is_none() {
        match (cmp, n) {
            (Comparison::Gt | Comparison::Geq, _) => {
                expected_status = Some(Status::RunFail);
            },
            (Comparison::Eq, n) if n != 0 => {
                expected_status = Some(Status::RunFail);
            },
            _ => {},
        }
    }

    Ok(Directive {
        expected_status: expected_status.unwrap_or(Status::RunPass),
        compile_error,
        compile_warning,
        run_error,
    })
}

fn check_compile_output(output: &subprocess::Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    let (compile_errors, compile_warnings) = match count_compile_errors_and_warnings(&String::from_utf8_lossy(&output.stderr)) {
        Some((e, w)) => (e, w),
        None => {
            return Err(String::from("failed to parse the compiler output"));
        },
    };

    match (output.status.success(), directive.expected_status) {
        (true, Status::CompileFail) => { return Err(String::from("expected compile-fail, but it passed")); },
        (false, Status::CompilePass | Status::RunFail | Status::RunPass) => { return Err(String::from("expected compile-pass, but it failed")); },
        _ => {},
    }

    if let Some((cmp, n)) = directive.compile_error {
        cmp.check(compile_errors, n, "the number of compile errors")?;
    }

    if let Some((cmp, n)) = directive.compile_warning {
        cmp.check(compile_warnings, n, "the number of compile warnings")?;
    }

    match_lines(&String::from_utf8_lossy(&output.stdout), &expected_output.compile_stdout).map_err(|e| format!("expected compile_stdout and actual stdout do not match\n{e}"))?;
    match_lines(&String::from_utf8_lossy(&output.stderr), &expected_output.compile_stderr).map_err(|e| format!("expected compile_stderr and actual stderr do not match\n{e}"))?;
    Ok(())
}

fn check_run_output(output: &subprocess::Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    match (output.status.success(), directive.expected_status) {
        (true, Status::RunFail) => { return Err(String::from("expected run-fail, but it passed")); },
        (false, Status::RunPass) => { return Err(String::from("expected run-pass, but if failed")); },
        _ => {},
    }

    if let Some((cmp, n)) = directive.run_error {
        todo!();
    }

    match_lines(&String::from_utf8_lossy(&output.stdout), &expected_output.run_stdout).map_err(|e| format!("expected run_stdout and actual stdout do not match\n{e}"))?;
    match_lines(&String::from_utf8_lossy(&output.stderr), &expected_output.run_stderr).map_err(|e| format!("expected run_stderr and actual stderr do not match\n{e}"))?;
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
