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

mod line_matcher;
mod tests;

pub use line_matcher::{LineMatcher, match_lines};
pub use tests::{Directive, ExpectedOutput, remove_ansi_characters};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompileAndRun {
    pub name: String,

    // test-runner generates this error message.
    pub error: Option<String>,
    pub stdout: String,
    pub stderr: String,

    // This has nothing to do with test pass/fail.
    // For example, if the test expects this case to compile-fail, but this case
    // successfully compiles and runs, it's `Status::RunPass` but it's an erroneous test.
    //
    // In order to check test pass/fail, you have to check whether the `.error` field is None.
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

impl Default for CompileAndRun {
    fn default() -> CompileAndRun {
        CompileAndRun {
            name: String::new(),
            error: None,
            stdout: String::new(),
            stderr: String::new(),
            status: Status::MiscError,
            stdout_colored: String::new(),
            stderr_colored: String::new(),
            hash: String::new(),
            compile_elapsed_ms: 0,
            run_elapsed_ms: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
    // Failed before `sodigy build`.
    // It could be a file-IO error, an error at `sodigy clean`, ... etc.
    MiscError,

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

struct CnrContext {
    pub name: String,
    pub root: String,
    pub sodigy_path: String,
    pub project_dir: String,
    pub expected_output: ExpectedOutput,
    pub sdg_files: usize,
    pub dump_output: bool,
}

pub fn run_cases(
    filter: Option<String>,
    root: &str,
    test_dir: &str,  // `<ROOT>/tests/compile-and-run/`
    sodigy_path: &str,
) -> Vec<CompileAndRun> {
    let mut cases = vec![];

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

        cases.push(case_name);
    }

    if let Some(filter) = &filter && cases.is_empty() {
        panic!("There's no test case that matches `{filter}`!")
    }

    // TODO: run tests in parallel?
    //       I'm not sure... because the sodigy compiler already runs in parallel,
    //       so I'm not sure how much performance gain I can get from parallelizing
    //       the tests.
    let mut result = vec![];
    let mut pass = 0;
    let mut fail = 0;

    for case in cases.iter() {
        let case_result = run_cnr(case, root, sodigy_path, filter.is_some());
        let (color, status) = if case_result.error.is_none() {
            pass += 1;
            (32, "pass")
        } else {
            fail += 1;
            (31, "fail")
        };

        println!("{case}: \x1b[{color}m{status}\x1b[0m");

        if filter.is_some() && let Some(error) = &case_result.error {
            eprintln!("{error}");
        }

        result.push(case_result);
    }

    println!("--------------------------");
    println!("pass: {pass}, fail: {fail}");
    result
}

fn run_cnr(
    name: &str,  // full name
    root: &str,
    sodigy_path: &str,
    dump_output: bool,
) -> CompileAndRun {
    let cnr_context = prepare_cnr(name, root, sodigy_path, dump_output);
    let mut result = cnr_context.main_test();
    cnr_context.extra_tests(&mut result);
    result
}

fn prepare_cnr(
    name: &str,
    root: &str,
    sodigy_path: &str,
    dump_output: bool,
) -> CnrContext {
    let test_dir = join3(root, "tests", "compile-and-run").unwrap();
    let base_path = join(&test_dir, name).unwrap();
    let test_file = set_extension(&base_path, "sdg").unwrap();
    let expected_output = ExpectedOutput {
        compile_stdout: parse_expected_output(&set_extension(&base_path, "compile.stdout").unwrap()),
        compile_stderr: parse_expected_output(&set_extension(&base_path, "compile.stderr").unwrap()),
        run_stdout: parse_expected_output(&set_extension(&base_path, "run.stdout").unwrap()),
        run_stderr: parse_expected_output(&set_extension(&base_path, "run.stderr").unwrap()),
    };
    let mut sdg_files = 1;

    let project_dir = if exists(&test_file) {
        println!("running `tests/compile-and-run/{name}.sdg`");
        let tmp_dir = create_tmp_project(&test_file, root, sodigy_path).unwrap();
        tmp_dir
    }

    else if exists(&base_path) && is_dir(&base_path) {
        // QoL: If it's a file and not a directory, text editor (ZED in my case) will
        //      open the file when I alt+click the path.
        println!("running `tests/compile-and-run/{name}/src/lib.sdg`");
        sdg_files = count_sdg_files(&base_path);
        base_path.clone()
    }

    else {
        panic!("No compile-and-run test case named `{name}`.")
    };

    CnrContext {
        name: name.to_string(),
        root: root.to_string(),
        sodigy_path: sodigy_path.to_string(),
        project_dir,
        expected_output,
        sdg_files,
        dump_output,
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

fn parse_expected_output(path: &str) -> Option<Vec<LineMatcher>> {
    if exists(path) {
        let s = read_string(path).unwrap();
        Some(s.lines().map(|line| LineMatcher::from_line(line)).collect())
    }

    else {
        None
    }
}

pub fn hash_dir(dir: &str) -> u128 {
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

fn count_sdg_files(dir: &str) -> usize {
    let mut result = 0;

    for file in read_dir(dir, false).unwrap() {
        if is_dir(&file) {
            result += count_sdg_files(&file);
        }

        else if file.ends_with(".sdg") {
            result += 1;
        }
    }

    result
}
