use sodigy_compiler_test::{
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
    write_string,
};

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
                suites: vec![TestSuite::Crates, TestSuite::CompileAndRun],
                crates,
                compile_and_run: compile_and_run_result,
            };
            let result = serde_json::to_string_pretty(&result).unwrap();

            write_string(&file_name, &result, WriteMode::CreateOrTruncate).unwrap();
            write_string(&log_path, &result, WriteMode::CreateOrTruncate).unwrap();
        },
        Some(_) => {},
        None => {},
    }
}
