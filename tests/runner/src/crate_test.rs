use serde::{Deserialize, Serialize};
use sodigy_fs_api::{file_name, read_dir};
use std::process::{Command, Output};
use std::time::Instant;

#[derive(Deserialize, Serialize)]
pub struct CrateTest {
    pub name: String,
    pub debug: CrateTestResult,
    pub release: CrateTestResult,
    pub doc: CrateTestResult,
}

#[derive(Deserialize, Serialize)]
pub struct CrateTestResult {
    // If there's an error, it stores the stderr of the test.
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

impl CrateTest {
    pub fn has_error(&self) -> bool {
        self.debug.has_error() ||
        self.release.has_error() ||
        self.doc.has_error()
    }
}

impl CrateTestResult {
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

pub fn run_all(dir: &str  /* of `/crates/` */) -> Vec<CrateTest> {
    let mut result = vec![];
    let mut pass = 0;
    let mut fail = 0;

    for crate_full_path in read_dir(dir, true).unwrap() {
        let crate_name = file_name(&crate_full_path).unwrap();
        println!("testing crates/{crate_name}");
        let case_result = run_case(&crate_name, &crate_full_path);

        if case_result.has_error() {
            fail += 1;
        }

        else {
            pass += 1;
        }

        result.push(case_result);
    }

    println!("--------------------------");
    println!("pass: {pass}, fail: {fail}");
    result
}

fn run_case(name: &str, path: &str) -> CrateTest {
    cargo_clean(path);

    let started_at = Instant::now();
    let debug = Command::new("cargo")
        .arg("test")
        .current_dir(path)
        .output()
        .unwrap();
    let debug = crate_test_result(debug, started_at);
    cargo_clean(path);

    let started_at = Instant::now();
    let release = Command::new("cargo")
        .arg("test")
        .arg("--release")
        .current_dir(path)
        .output()
        .unwrap();
    let release = crate_test_result(release, started_at);
    cargo_clean(path);

    let started_at = Instant::now();
    let doc = Command::new("cargo")
        .arg("doc")
        .current_dir(path)
        .output()
        .unwrap();
    let doc = crate_test_result(doc, started_at);
    cargo_clean(path);

    CrateTest {
        name: name.to_string(),
        debug,
        release,
        doc,
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

fn crate_test_result(output: Output, started_at: Instant) -> CrateTestResult {
    let elapsed_ms = Instant::now().duration_since(started_at).as_millis() as u64;
    let error = if output.status.success() {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    CrateTestResult { error, elapsed_ms }
}
