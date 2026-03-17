use sodigy_compiler_test::{
    Fuzzer,
    FuzzTarget,
    TestHarness,
    TestSuite,
    compile_and_run,
    crate_test,
    find_root,
    get_sodigy_path,
    meta,
};
use sodigy_fs_api::{
    WriteMode,
    create_dir,
    exists,
    join,
    join3,
    join4,
    parent,
    write_bytes,
    write_string,
};
use std::thread;
use std::time::Duration;

// TODO: add fuzzer to the pipeline
//       1. switch to nightly rustc before invoking fuzzer, then come back to the stable rustc
//       2. do not run fuzzer on windows
//       3. if it's `all` command,
//          a. initiate the fuzzer before anything else
//          b. run the tests (the fuzzer is running along background)
//          c. the tests will run at least a few minutes, so the fuzzer has enough time to fuzz
//          d. when the tests are complete, kill the fuzzer process and check if there's a new error
//
// TODO: copy `.sdg` files in `compile-and-run/` to `../../fuzz/artifacts/fuzz_target_1/`
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let root = find_root().unwrap();
    let sodigy_path = get_sodigy_path(&root);

    match args.get(1).map(|arg| arg.as_str()) {
        Some("cnr") => {
            compile_and_run::run_cases(
                args.get(2).map(|arg| arg.to_string()),
                &root,
                &join3(&root, "tests", "compile-and-run").unwrap(),
                &sodigy_path,
            );
        },
        Some("crates") => {
            crate_test::run_all(&join(&root, "crates").unwrap());
        },
        Some("fuzz") => {
            let timeout = 300;
            let fuzz_dir = join(&root, "fuzz").unwrap();
            let cnr_dir = join3(&root, "tests", "compile-and-run").unwrap();
            let mut empty_fuzzer = Fuzzer::init(&fuzz_dir, &cnr_dir, FuzzTarget::Empty, false);
            let mut cnr_fuzzer = Fuzzer::init(&fuzz_dir, &cnr_dir, FuzzTarget::Cnr, false);

            for _ in 0..timeout {
                if let Some(fuzz_result) = empty_fuzzer.try_collect() {
                    if let Some(artifact) = &fuzz_result.artifact {
                        write_bytes(
                            "fuzz-empty.sdg",
                            artifact,
                            WriteMode::CreateOrTruncate,
                        ).unwrap();
                    }
                    break;
                }

                thread::sleep(Duration::from_millis(1_000));
            }

            for _ in 0..timeout {
                if let Some(fuzz_result) = cnr_fuzzer.try_collect() {
                    if let Some(artifact) = &fuzz_result.artifact {
                        write_bytes(
                            "fuzz-cnr.sdg",
                            artifact,
                            WriteMode::CreateOrTruncate,
                        ).unwrap();
                    }
                    break;
                }

                thread::sleep(Duration::from_millis(1_000));
            }
        },
        Some("all") => {
            let metadata = meta::get();

            if !metadata.is_repo_clean {
                println!("@@@@@@@");
                println!("WARNING: The repository is dirty!!");
                println!("Please commit changes before running the tests.");
                println!("@@@@@@@");
            }

            let fuzz_dir = join(&root, "fuzz").unwrap();
            let cnr_dir = join3(&root, "tests", "compile-and-run").unwrap();
            let mut empty_fuzzer = Fuzzer::init(&fuzz_dir, &cnr_dir, FuzzTarget::Empty, true);
            let mut cnr_fuzzer = Fuzzer::init(&fuzz_dir, &cnr_dir, FuzzTarget::Cnr, true);

            let crates = Some(crate_test::run_all(&join(&root, "crates").unwrap()));
            let compile_and_run_result = Some(compile_and_run::run_cases(
                None,
                &root,
                &join3(&root, "tests", "compile-and-run").unwrap(),
                &sodigy_path,
            ));
            let empty_fuzz_result = empty_fuzzer.collect();
            let cnr_fuzz_result = cnr_fuzzer.collect();

            let file_name = metadata.get_result_file_name();
            let log_path = join4(
                &root,
                "tests",
                "log",
                &file_name,
            ).unwrap();

            if !exists(&parent(&log_path).unwrap()) {
                create_dir(&parent(&log_path).unwrap()).unwrap();
            }

            let result = TestHarness {
                meta: metadata,
                suites: vec![TestSuite::Crates, TestSuite::CompileAndRun, TestSuite::Fuzz],
                crates,
                compile_and_run: compile_and_run_result,
                fuzz: Some(vec![empty_fuzz_result, cnr_fuzz_result]),
            };
            let result = serde_json::to_string_pretty(&result).unwrap();

            write_string(&file_name, &result, WriteMode::CreateOrTruncate).unwrap();
            write_string(&log_path, &result, WriteMode::CreateOrTruncate).unwrap();
        },
        Some(_) => {},
        None => {},
    }
}
