use serde::{Deserialize, Serialize};
use sodigy_fs_api::{file_name, read_dir};
use std::process::{Command, Output};
use std::time::Instant;

#[derive(Clone, Deserialize, Serialize)]
pub struct CrateTest {
    pub name: String,
    pub tests: Vec<CrateTestUnit>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrateTestUnit {
    pub command: String,
    pub has_error: bool,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u64,
}

impl CrateTest {
    pub fn has_error(&self) -> bool {
        self.tests.iter().any(|t| t.has_error)
    }

    pub fn summary(&self) -> String {
        self.tests.iter().map(
            |t| format!(
                "`cargo {}`: {}",
                if t.command.len() < 32 {
                    t.command.to_string()
                } else {
                    format!("{}...", t.command.get(0..29).unwrap())
                },
                if t.has_error { "fail" } else { "true" },
            )
        ).collect::<Vec<_>>().join("\n")
    }
}

impl CrateTestUnit {
    pub fn dump_error(&self) -> Option<&String> {
        if self.has_error {
            Some(&self.stderr)
        } else {
            None
        }
    }
}

pub fn run_cases(
    dir: &str,  /* of `/crates/` */
    filter: Option<Vec<String>>,
    verbose: bool,
) -> Vec<CrateTest> {
    let mut result = vec![];
    let mut pass = 0;
    let mut fail = 0;

    for crate_full_path in read_dir(dir, true).unwrap() {
        let crate_name = file_name(&crate_full_path).unwrap();

        if let Some(filter) = &filter {
            if !filter.contains(&crate_name) {
                continue;
            }
        }

        println!("testing crates/{crate_name}");
        let case_result = run_case(&crate_name, &crate_full_path, verbose);

        let (color, status) = if case_result.has_error() {
            fail += 1;
            (31, "fail")
        }

        else {
            pass += 1;
            (32, "pass")
        };

        println!("{crate_name}: \x1b[{color}m{status}\x1b[0m");
        result.push(case_result);
    }

    println!("--------------------------");
    println!("pass: {pass}, fail: {fail}");
    result
}

fn run_case(name: &str, path: &str, verbose: bool) -> CrateTest {
    let mut tests = vec![];
    cargo_clean(path);
    let mut clippy_args = vec![String::from("clippy"), String::from("--")];

    for (name, level) in [
        ("mismatched_lifetime_syntaxes", "-D"),
        ("suspicious_double_ref_op", "-D"),
        ("unknown_lints", "-D"),
        ("unreachable_patterns", "-D"),
        ("unused_crate_dependencies", "-D"),
        ("unused_imports", "-D"),
        ("unused_mut", "-D"),
        ("clippy::cast_abs_to_unsigned", "-D"),
        ("clippy::clone_on_copy", "-D"),
        ("clippy::cloned_ref_to_slice_refs", "-D"),
        ("clippy::drain_collect", "-D"),
        ("clippy::let_and_return", "-D"),
        ("clippy::manual_retain", "-D"),
        ("clippy::map_clone", "-D"),
        ("clippy::needless_borrow", "-D"),
        ("clippy::needless_bool", "-D"),
        ("clippy::nonminimal_bool", "-D"),
        ("clippy::only_used_in_recursion", "-D"),
        ("clippy::op_ref", "-D"),
        ("clippy::redundant_field_names", "-D"),
        ("clippy::redundant_static_lifetimes", "-D"),
        ("clippy::replace_box", "-D"),
        ("clippy::unnecessary_cast", "-D"),
        ("clippy::unnecessary_get_then_check", "-D"),
        ("clippy::unnecessary_unwrap", "-D"),
        ("clippy::unneeded_struct_pattern", "-D"),
        ("clippy::useless_conversion", "-D"),

        // I'll deny this eventually, but not now.
        // Sodigy is still under heavy development and there are so many unused variables.
        ("unused_variables", "-A"),

        // I'm not sure about these. I just allowed these so that stderr doesn't get bloated.
        // I might change my mind in the future...
        ("clippy::collapsible_if", "-A"),
        ("clippy::collapsible_else_if", "-A"),
        ("clippy::extend_with_drain", "-A"),
        ("clippy::derivable_impls", "-A"),
        ("clippy::manual_flatten", "-A"),
        ("clippy::needless_return", "-A"),
        ("clippy::redundant_closure", "-A"),
        ("clippy::redundant_pattern_matching", "-A"),
        ("clippy::result_large_err", "-A"),
        ("clippy::single_match", "-A"),
        ("clippy::too_many_arguments", "-A"),
        ("clippy::useless_vec", "-A"),

        // I'll always allow these.
        ("clippy::get_first", "-A"),        // If `x.get(0)` should be `x.first()`, then `x.get(1)` should be `x.second()` and `x.get(12)` should be `x.thirteenth()`, right?
        ("clippy::iter_kv_map", "-A"),      // I don't want to import HashMap just for this reason.
        ("clippy::len_zero", "-A"),         // I think `x.len() > 0` is more readable than `!x.is_empty()`
        ("clippy::result_unit_err", "-A"),  // I know it's a bad pattern, but it's necessary in Sodigy. It requires some kinda deferred error handling.
        ("clippy::type_complexity", "-A"),  // I decide how complex a type is...
    ] {
        clippy_args.push(level.to_string());
        clippy_args.push(name.to_string());
    }

    let started_at = Instant::now();
    let output = Command::new("cargo")
        .args(clippy_args.clone())
        .current_dir(path)
        .output()
        .unwrap();
    let result = crate_test_unit(&clippy_args, output, started_at);

    if verbose && let Some(error) = result.dump_error() {
        println!("--- clippy stderr ---");
        println!("{error}");
    }

    tests.push(result);
    cargo_clean(path);

    let started_at = Instant::now();
    let output = Command::new("cargo")
        .arg("doc")
        .current_dir(path)
        .output()
        .unwrap();
    let result = crate_test_unit(&[String::from("doc")], output, started_at);

    if verbose && let Some(error) = result.dump_error() {
        println!("--- doc stderr ---");
        println!("{error}");
    }

    tests.push(result);
    cargo_clean(path);

    let started_at = Instant::now();
    let output = Command::new("cargo")
        .arg("test")
        .current_dir(path)
        .output()
        .unwrap();
    let result = crate_test_unit(&[String::from("test")], output, started_at);

    if verbose && let Some(error) = result.dump_error() {
        println!("--- debug stderr ---");
        println!("{error}");
    }

    tests.push(result);
    cargo_clean(path);

    let started_at = Instant::now();
    let output = Command::new("cargo")
        .arg("test")
        .arg("--release")
        .current_dir(path)
        .output()
        .unwrap();
    let result = crate_test_unit(&[String::from("test"), String::from("--release")], output, started_at);

    if verbose && let Some(error) = result.dump_error() {
        println!("--- release stderr ---");
        println!("{error}");
    }

    tests.push(result);
    cargo_clean(path);

    CrateTest {
        name: name.to_string(),
        tests,
    }
}

fn cargo_clean(path: &str) {
    let output = Command::new("cargo")
        .arg("clean")
        .current_dir(path)
        .output()
        .unwrap();
    assert!(output.status.success());
}

fn crate_test_unit(command: &[String], output: Output, started_at: Instant) -> CrateTestUnit {
    let elapsed_ms = Instant::now().duration_since(started_at).as_millis() as u64;
    let has_error = !output.status.success();

    CrateTestUnit {
        command: command.join(" "),
        has_error,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        elapsed_ms,
    }
}

