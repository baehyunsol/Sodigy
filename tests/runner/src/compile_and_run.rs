use crate::subprocess::{self, SubprocessError};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    exists,
    file_name,
    is_dir,
    join,
    join3,
    join4,
    read_dir,
    read_string,
    remove_dir_all,
    set_extension,
    write_string,
};
use sodigy_string::hash;
use std::process::Command;
use std::time::Instant;

mod line_matcher;

pub use line_matcher::{LineMatcher, match_lines};

#[derive(Deserialize, Serialize)]
pub struct CompileAndRun {
    pub name: String,

    // test-runner generates this error message.
    pub error: Option<String>,
    pub stdout: String,
    pub stderr: String,

    // This has nothing to do with test pass/fail.
    // For example, if the test expects this case to compile-fail, but this case
    // successfully compiles and runs, it's `Status::RunPass` but it's an erroneous test.
    pub status: Status,

    // Uses ANSI-terminal colors.
    pub stdout_colored: String,
    pub stderr_colored: String,

    // Hash of the test file(s).
    pub hash: String,

    pub compile_elapsed_ms: u64,

    // It's None if the compilation failed.
    pub run_elapsed_ms: Option<u64>,
}

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
    CompileTimeout,
    CompileFail,

    // This has 2 use cases.
    // 1. The directive wants the test runner to make sure that the compile passes, but doesn't care about the run pass/fail.
    // 2. The runner checked that the compile passes, but didn't run it.
    CompilePass,

    // It implies `CompilePass`.
    RunTimeout,
    RunFail,
    RunPass,
}

pub fn run_with_condition(
    filter: Option<String>,
    root: &str,
    test_dir: &str,  // of `<ROOT>/tests/compile-and-run/`
    sodigy_path: &str,
) -> Vec<CompileAndRun> {
    let mut result = vec![];
    let mut pass = 0;
    let mut fail = 0;

    for file in read_dir(test_dir, true).unwrap() {
        if let Some(case_name) = file.strip_suffix(".compile.stdout").or_else(
            || file.strip_suffix(".compile.stderr")
        ).or_else(
            || file.strip_suffix(".run.stdout")
        ).or_else(
            || file.strip_suffix(".run.stderr")
        ) {
            if !exists(&set_extension(case_name, "sdg").unwrap()) && !is_dir(case_name) {
                panic!(
                    "There's an expected output for test case `{}`, but there's no test case that's named `{}`.",
                    file_name(case_name).unwrap(),
                    file_name(case_name).unwrap(),
                );
            }

            continue;
        }

        else if file.ends_with(".sdg") || is_dir(&file) {}

        else {
            panic!("Unknown file kind: `{file}`.");
        }

        let case_name = file_name(&file).unwrap();

        if let Some(filter) = &filter {
            let condition = match filter {
                _ if filter.starts_with("^") && filter.ends_with("$") => case_name == filter.get(1..(filter.len() - 1)).unwrap(),
                _ if filter.starts_with("^") => case_name.starts_with(filter.get(1..).unwrap()),
                _ if filter.ends_with("$") => case_name.ends_with(filter.get(..(filter.len() - 1)).unwrap()),
                _ => case_name.contains(filter),
            };

            if !condition {
                continue;
            }
        }

        let case_result = run_case(&case_name, root, sodigy_path, filter.is_some());
        let (color, status) = if case_result.error.is_none() {
            pass += 1;
            (32, "pass")
        } else {
            fail += 1;
            (31, "fail")
        };

        println!("{case_name}: \x1b[{color}m{status}\x1b[0m");
        result.push(case_result);
    }

    if let Some(filter) = filter && result.is_empty() {
        panic!("There's no test case that matches `{filter}`!")
    }

    println!("--------------------------");
    println!("pass: {pass}, fail: {fail}");
    result
}

fn run_case(
    name: &str,  // full name
    root: &str,
    sodigy_path: &str,
    dump_output: bool,
) -> CompileAndRun {
    let test_dir = join3(root, "tests", "compile-and-run").unwrap();
    let base_path = join(&test_dir, name).unwrap();
    let test_file = set_extension(&base_path, "sdg").unwrap();
    let expected_output = ExpectedOutput {
        compile_stdout: parse_expected_output(&set_extension(&base_path, "compile.stdout").unwrap()),
        compile_stderr: parse_expected_output(&set_extension(&base_path, "compile.stderr").unwrap()),
        run_stdout: parse_expected_output(&set_extension(&base_path, "run.stdout").unwrap()),
        run_stderr: parse_expected_output(&set_extension(&base_path, "run.stderr").unwrap()),
    };

    let project_dir = if exists(&test_file) {
        println!("running `tests/compile-and-run/{name}.sdg`");
        let tmp_dir = create_tmp_project(&test_file, root, sodigy_path).unwrap();
        tmp_dir
    }

    else if exists(&base_path) && is_dir(&base_path) {
        // QoL: If it's a file and not a directory, text editor (ZED in my case) will
        //      open the file when I alt+click the path.
        println!("running `tests/compile-and-run/{name}/src/lib.sdg`");
        base_path.clone()
    }

    else {
        panic!("No compile-and-run test case named `{name}`.")
    };

    run_case_inner(name, &project_dir, &expected_output, sodigy_path, dump_output)
}

fn run_case_inner(
    name: &str,
    project_dir: &str,
    expected_output: &ExpectedOutput,
    sodigy_path: &str,
    dump_output: bool,
) -> CompileAndRun {
    let lib_src = join3(project_dir, "src", "lib.sdg").unwrap();
    let directive = parse_directive(&lib_src).unwrap();
    let mut stdout_colored = vec![];
    let mut stderr_colored = vec![];

    if directive.expected_status == Status::CompileFail && expected_output.compile_stderr.is_none() {
        panic!("If you want to assert that `{name}` fails to compile, please add `{name}.compile.stderr` file.");
    }

    // TODO: there may be ir_dir from previous test run -> remove it
    // TODO: do we have to hash expected_output?
    let hash = format!("{:024x}", hash_dir(&join(project_dir, "src").unwrap()));
    let compile_started_at = Instant::now();
    let output = match subprocess::run(
        sodigy_path,
        &["build", "--test", "-o=target/run"],
        project_dir,
        30.0,
        dump_output,
    ) {
        Ok(output) => output,
        Err(SubprocessError::Timeout) => {
            return CompileAndRun {
                name: name.to_string(),
                error: Some(String::from("compile-timeout")),
                stdout: String::new(),
                stderr: String::new(),
                status: Status::CompileTimeout,
                stdout_colored: String::new(),
                stderr_colored: String::new(),
                hash,
                compile_elapsed_ms: Instant::now().duration_since(compile_started_at).as_millis() as u64,
                run_elapsed_ms: None,
            };
        },
        Err(e) => panic!("{e:?}"),
    };

    let compile_elapsed_ms = Instant::now().duration_since(compile_started_at).as_millis() as u64;
    stdout_colored.extend(&output.stdout);
    stderr_colored.extend(&output.stderr);

    let mut error = match check_compile_output(&output, &directive, expected_output) {
        Ok(()) => None,
        Err(e) => Some(e),
    };
    let mut status = if output.status.success() { Status::CompilePass } else { Status::CompileFail };
    let mut run_elapsed_ms = None;

    if status != Status::CompileFail {
        let run_started_at = Instant::now();
        match subprocess::run(
            sodigy_path,
            &["interpret", "target/run"],
            project_dir,
            30.0,
            dump_output,
        ) {
            Ok(output) => {
                run_elapsed_ms = Some(Instant::now().duration_since(run_started_at).as_millis() as u64);
                stdout_colored.extend(&output.stdout);
                stderr_colored.extend(&output.stderr);

                error = match (error, check_run_output(&output, &directive, expected_output)) {
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
            Err(e) => panic!("{e:?}"),
        }
    }

    let stdout_colored = String::from_utf8_lossy(&stdout_colored).to_string();
    let stderr_colored = String::from_utf8_lossy(&stderr_colored).to_string();

    CompileAndRun {
        name: name.to_string(),
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

fn create_tmp_project(
    file: &str,
    root: &str,
    sodigy_path: &str,
) -> Result<String, FileError> {
    if exists(&join(root, "sodigy-test")?) {
        remove_dir_all(&join(root, "sodigy-test")?)?;
    }

    let file_content = read_string(file)?;
    let output = Command::new(sodigy_path)
        .arg("new")
        .arg("sodigy-test")
        .current_dir(root)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(output.success());
    write_string(
        &join4(
            root,
            "sodigy-test",
            "src",
            "lib.sdg",
        )?,
        &file_content,
        WriteMode::CreateOrTruncate,
    )?;
    Ok(join(root, "sodigy-test")?)
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

fn parse_expected_output(path: &str) -> Option<Vec<LineMatcher>> {
    if exists(path) {
        let s = read_string(path).unwrap();
        Some(s.lines().map(|line| LineMatcher::from_line(line)).collect())
    }

    else {
        None
    }
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

#[derive(Clone, Copy)]
enum AnsiParseState {
    Text,
    Escape,
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

fn hash_dir(dir: &str) -> u128 {
    let mut sum = 0;

    for f in read_dir(dir, true).unwrap() {
        if is_dir(&f) {
            sum += hash_dir(&f);
        }

        else {
            let s = read_string(&f).unwrap();
            sum += hash(s.as_bytes()) & 0xffff_ffff_ffff_ffff_ffff_ffff;
        }
    }

    sum & 0xffff_ffff_ffff_ffff_ffff_ffff
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
