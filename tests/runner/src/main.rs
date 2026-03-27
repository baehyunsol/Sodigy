use sodigy_cli::{
    ArgCount,
    ArgParser,
    ArgType,
};
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

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let root = find_root().unwrap();

    match args.get(1).map(|arg| arg.as_str()) {
        Some("cnr") => {
            let parsed_args = ArgParser::new()
                .optional_flag(&["--log-inter-mir"])
                .optional_flag(&["--log-post-mir"])
                .args(ArgType::String, ArgCount::Leq(1))
                .parse(&args, 2)
                .map_err(|_| "cli error")
                .unwrap();

            let log_inter_mir = parsed_args.get_flag(0).is_some();
            let log_post_mir = parsed_args.get_flag(1).is_some();
            let filter = parsed_args.get_args().get(0).map(|f| f.to_string());
            let sodigy_path = get_sodigy_path(
                &root,
                false,  // --release
                log_inter_mir,
            );

            compile_and_run::run_cases(
                filter,
                &root,
                &join3(&root, "tests", "compile-and-run").unwrap(),
                &sodigy_path,
                log_post_mir,
            );
        },
        Some("crates") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Geq(0))
                .parse(&args, 2)
                .map_err(|_| "cli error")
                .unwrap();

            let filter = {
                let args = parsed_args.get_args();

                if args.is_empty() {
                    None
                } else {
                    Some(args.iter().map(|f| f.to_string()).collect())
                }
            };
            let crates_at = join(&root, "crates").unwrap();
            crate_test::run_cases(&crates_at, filter, true);
        },
        Some("fuzz") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--timeout", ArgType::integer_between(Some(0), Some(u32::MAX.into())))
                .flag_with_default(&["--all", "--empty", "--cnr"])
                .parse(&args, 2)
                .map_err(|_| "cli error")
                .unwrap();

            let timeout = parsed_args.arg_flags.get("--timeout").map(
                |n| n.parse::<usize>().unwrap()
            ).unwrap_or(300);
            let (cnr, empty) = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--all") => (true, true),
                Some("--cnr") => (true, false),
                Some("--empty") => (false, true),
                _ => unreachable!(),
            };

            let fuzz_dir = join(&root, "fuzz").unwrap();
            let cnr_dir = join3(&root, "tests", "compile-and-run").unwrap();

            if empty {
                let mut fuzzer = Fuzzer::init(&fuzz_dir, &cnr_dir, FuzzTarget::Empty, false);

                for _ in 0..timeout {
                    if let Some(fuzz_result) = fuzzer.try_collect() {
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
            }

            if cnr {
                let mut fuzzer = Fuzzer::init(&fuzz_dir, &cnr_dir, FuzzTarget::Cnr, false);

                for _ in 0..timeout {
                    if let Some(fuzz_result) = fuzzer.try_collect() {
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
            }
        },
        Some("all") => {
            let sodigy_path = get_sodigy_path(
                &root,
                false,  // --release
                false,  // log-inter-mir
            );

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

            let crates_at = join(&root, "crates").unwrap();
            let crates = Some(crate_test::run_cases(&crates_at, None, false));
            let compile_and_run_result = Some(compile_and_run::run_cases(
                None,
                &root,
                &join3(&root, "tests", "compile-and-run").unwrap(),
                &sodigy_path,
                false,
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
        Some(_) => todo!(),
        None => todo!(),
    }
}
