use sodigy_fs_api::{
    FileError,
    basename,
    exists,
    into_abs_path,
    join,
    join4,
    read_dir,
};
use std::process::Command;

pub mod compile_and_run;
pub mod crate_test;
pub mod fuzzer;
pub mod harness;
pub mod meta;
pub mod subprocess;

pub use compile_and_run::{CompileAndRun, hash_dir};
pub use crate_test::CrateTest;
pub use fuzzer::{FuzzResult, FuzzTarget, Fuzzer};
pub use harness::{TestHarness, TestSuite};
pub use meta::{Meta, git};

// If it fails to compile the sodigy-compiler, it panics.
// It doesn't capture stderr.
pub fn get_sodigy_path(
    root: &str,
    release: bool,
    log_inter_mir: bool,
) -> String {
    let mut args = vec!["build"];

    if release {
        args.push("--release");
    }

    if log_inter_mir {
        args.push("--features=log-inter-mir");
    }

    let output = Command::new("cargo")
        .args(&args)
        .current_dir(root)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let path = join4(
        root,
        "target",
        if release { "release" } else { "debug" },
        "sodigy",
    ).unwrap();
    assert!(exists(&path));
    assert!(output.success(), "failed to build sodigy-compiler");

    into_abs_path(&path).unwrap()
}

pub fn find_root() -> Result<String, FileError> {
    let mut pwd = String::from(".");

    // In some OSes, it might loop forever if there's an error
    for _ in 0..256 {
        let mut dir = read_dir(&pwd, false)?;
        dir = dir.iter().map(|f| basename(f).unwrap()).collect();

        if dir.contains(&String::from("crates")) && dir.contains(&String::from("Cargo.toml")) {
            return Ok(pwd);
        }

        pwd = join(&pwd, "..")?;
    }

    panic!()
}
