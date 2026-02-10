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
use std::process::{Command, Output};
use std::time::Instant;

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

    // Hash of the test file.
    // It uses `git hash-object` to hash the file.
    // TODO: what if there are multiple files?
    // TODO: let's just use my own hash function...
    pub hash: String,

    pub compile_elapsed_ms: u64,

    // It's None if the compilation failed.
    pub run_elapsed_ms: Option<u64>,
}

pub struct ExpectedOutput {
    pub compile_stdout: Option<Vec<LineMatch>>,
    pub compile_stderr: Option<Vec<LineMatch>>,
    pub run_stdout: Option<Vec<LineMatch>>,
    pub run_stderr: Option<Vec<LineMatch>>,
}

enum LineMatch {
    AnyLines,
    Exact(String),
    // TODO: matches with `...`
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
    CompileFail,

    // This has 2 use cases.
    // 1. The directive wants the test runner to make sure that the compile passes, but doesn't care about the run pass/fail.
    // 2. The runner checked that the compile passes, but didn't run it.
    CompilePass,

    // It implies `CompilePass`.
    RunFail,
    RunPass,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Comparison {
    Gt,
    Geq,
    Lt,
    Leq,
    Eq,
    Neq,
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
        }

        else if file.ends_with(".sdg") || is_dir(&file) {}

        else {
            panic!("Unknown file kind: `{file}`.");
        }

        let case_name = file_name(&file).unwrap();

        if let Some(filter) = &filter {
            if !case_name.contains(filter) {
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
        compile_stdout: parse_expected_output(&join(&base_path, ".compile.stdout").unwrap()),
        compile_stderr: parse_expected_output(&join(&base_path, ".compile.stderr").unwrap()),
        run_stdout: parse_expected_output(&join(&base_path, ".run.stdout").unwrap()),
        run_stderr: parse_expected_output(&join(&base_path, ".run.stderr").unwrap()),
    };

    let project_dir = if exists(&test_file) {
        println!("running `tests/compile-and-run/{name}.sdg`");
        let tmp_dir = create_tmp_project(&test_file, root, sodigy_path).unwrap();
        tmp_dir
    }

    else if exists(&base_path) && is_dir(&base_path) {
        println!("running `tests/compile-and-run/{name}/`");
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

    // TODO: do we have to hash expected_output?
    let hash = format!("{:024x}", hash_dir(&join(project_dir, "src").unwrap()));
    let compile_started_at = Instant::now();

    let output = Command::new(sodigy_path)
        .arg("build")
        .arg("--test")
        .arg("-o=main")
        .current_dir(project_dir)
        .output()
        .unwrap();
    let compile_elapsed_ms = Instant::now().duration_since(compile_started_at).as_millis() as u64;
    stdout_colored.extend(&output.stdout);
    stderr_colored.extend(&output.stderr);

    if dump_output {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    let mut error = match check_compile_output(&output, &directive, expected_output) {
        Ok(()) => None,
        Err(e) => Some(e),
    };
    let mut status = if output.status.success() { Status::CompilePass } else { Status::CompileFail };
    let mut run_elapsed_ms = None;

    if status != Status::CompileFail {
        let run_started_at = Instant::now();

        let output = Command::new(sodigy_path)
            .arg("interpret")
            .arg("main")
            .current_dir(project_dir)
            .output()
            .unwrap();
        run_elapsed_ms = Some(Instant::now().duration_since(run_started_at).as_millis() as u64);
        stdout_colored.extend(&output.stdout);
        stderr_colored.extend(&output.stderr);

        if dump_output {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        error = match (error, check_run_output(&output, &directive, expected_output)) {
            (None, Err(e)) => Some(e),
            (e, _) => e,
        };

        status = if output.status.success() { Status::RunPass } else { Status::RunFail };
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

    Ok(Directive {
        expected_status: expected_status.unwrap_or(Status::RunPass),
        compile_error,
        compile_warning,
        run_error,
    })
}

fn parse_expected_output(path: &str) -> Option<Vec<LineMatch>> {
    if exists(path) {
        todo!()
    }

    else {
        None
    }
}

fn check_compile_output(output: &Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    match (output.status.success(), directive.expected_status) {
        (true, Status::CompileFail) => { return Err(String::from("expected compile-fail, but it passed")); },
        (false, Status::CompilePass | Status::RunFail | Status::RunPass) => { return Err(String::from("expected compile-pass, but it failed")); },
        _ => {},
    }

    if let Some((cmp, n)) = directive.compile_error {
        todo!();
    }

    if let Some((cmp, n)) = directive.compile_warning {
        todo!();
    }

    match_lines(&output.stdout, &expected_output.compile_stdout)?;
    match_lines(&output.stderr, &expected_output.compile_stderr)?;
    Ok(())
}

fn check_run_output(output: &Output, directive: &Directive, expected_output: &ExpectedOutput) -> Result<(), String> {
    match (output.status.success(), directive.expected_status) {
        (true, Status::RunFail) => { return Err(String::from("expected run-fail, but it passed")); },
        (false, Status::RunPass) => { return Err(String::from("expected run-pass, but if failed")); },
        _ => {},
    }

    if let Some((cmp, n)) = directive.run_error {
        todo!();
    }

    match_lines(&output.stdout, &expected_output.run_stdout)?;
    match_lines(&output.stderr, &expected_output.run_stderr)?;
    Ok(())
}

#[derive(Clone, Copy)]
enum AnsiParseState {
    Text,
    Escape,
}

fn remove_ansi_characters(s: &str) -> String {
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

fn match_lines(s: &[u8], lines: &Option<Vec<LineMatch>>) -> Result<(), String> {
    if let Some(lines) = lines {
        todo!()
    }

    else {
        Ok(())
    }
}
