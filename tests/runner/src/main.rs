use sodigy_fs_api::{
    FileError,
    WriteMode,
    basename,
    exists,
    into_abs_path,
    join,
    join3,
    join4,
    read_dir,
    write_string,
};
use std::process::Command;

mod compile_and_run;
mod crate_test;
mod harness;
mod meta;

pub use compile_and_run::CompileAndRun;
pub use crate_test::CrateTest;
pub use harness::{TestHarness, TestSuite};
pub use meta::Meta;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let root = find_root().unwrap();
    let sodigy_path = get_sodigy_path(&root);

    match args.get(1).map(|arg| arg.as_str()) {
        Some("cnr") => {
            compile_and_run::run_with_condition(
                args.get(2).map(|arg| arg.to_string()),
                &root,
                &join3(&root, "tests", "compile-and-run").unwrap(),
                &sodigy_path,
            );
        },
        Some("crates") => {
            crate_test::run_all(&join(&root, "crates").unwrap());
        },
        Some("all") => {
            let metadata = meta::get();

            if !metadata.is_repo_clean {
                println!("@@@@@@@");
                println!("WARNING: The repository is dirty!!");
                println!("Please commit changes before running the tests.");
                println!("@@@@@@@");
            }

            let crates = Some(crate_test::run_all(&join(&root, "crates").unwrap()));
            let compile_and_run_result = Some(compile_and_run::run_with_condition(
                None,
                &root,
                &join3(&root, "tests", "compile-and-run").unwrap(),
                &sodigy_path,
            ));
            let file_name = metadata.get_result_file_name();
            let result = TestHarness {
                meta: metadata,
                suites: vec![TestSuite::Crates, TestSuite::CompileAndRun],
                crates,
                compile_and_run: compile_and_run_result,
            };

            write_string(
                &file_name,
                &serde_json::to_string_pretty(&result).unwrap(),
                WriteMode::CreateOrTruncate,
            ).unwrap();
        },
        Some(_) => {},
        None => {},
    }
}

// If it fails to compile the sodigy-compiler, it panics.
// It doesn't capture stderr.
pub fn get_sodigy_path(root: &str) -> String {
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(root)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let path = join4(
        root,
        "target",
        "debug",
        "sodigy",
    ).unwrap();
    assert!(exists(&path));
    assert!(output.success());

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
