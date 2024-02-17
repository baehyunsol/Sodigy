use crate::{CompilerOutput, SAVE_IRS_AT, clean_irs, run};
use json::JsonValue;
use sodigy_clap::{CompilerOption, CompilerOutputFormat};
use sodigy_files::{
    create_dir,
    exists,
    join,
    parent,
    read_bytes,
    read_string,
    remove_dir_all,
    remove_file,
};
use std::sync::Mutex;

// `clean_irs` must not be interleaved
static mut LOCK: Mutex<()> = Mutex::new(());

type Path = String;

// If this test panics, it leaves tmp files. You MUST run `nu clean.nu` to remove the tmp files.
//
// 1. compile the file directly from code to the end
// 2. compile the file using intermediate result from 1
// 3. compile the file from code to hir, then from hir to the end
// 4. dump mirs of 1, 2 and 3. make sure that they're identical
fn test_runner1(path: &str) {
    let lock = unsafe { LOCK.lock().unwrap() };

    let mut dummy_compiler_output = CompilerOutput::new();
    let dir_to_clean = parent(path).unwrap();
    let tmp_result_dir = join(".", &format!("__tmp_{:x}", rand::random::<u128>())).unwrap();
    create_dir(&tmp_result_dir).unwrap();

    clean_irs(&dir_to_clean, &mut dummy_compiler_output, &mut 0);

    let bin_outputs = (0..3).map(
        |i| join(&tmp_result_dir, &format!("{i}.out")).unwrap()
    ).collect::<Vec<_>>();
    let hir_outputs = (0..2).map(
        |i| join(&tmp_result_dir, &format!("{i}.hir.json")).unwrap()
    ).collect::<Vec<_>>();
    let mir_outputs = (0..2).map(
        |i| join(&tmp_result_dir, &format!("{i}.mir.json")).unwrap()
    ).collect::<Vec<_>>();

    let base_option = CompilerOption {
        input_file: Some(path.to_string()),
        do_not_compile_and_do_this: None,
        show_warnings: true,
        save_ir: true,
        dump_hir_to: None,
        dump_mir_to: None,
        ..CompilerOption::default()
    };

    let compile_option1 = CompilerOption {
        output: CompilerOutputFormat::Path(bin_outputs[0].clone()),
        dump_hir_to: Some(hir_outputs[0].clone()),
        dump_mir_to: Some(mir_outputs[0].clone()),
        ..base_option.clone()
    };

    // 1. end to end compile (clean)
    let mut res = run(compile_option1);

    if res.has_error() {
        panic!("{}", res.concat_results());
    }

    // compilation from cached data doesn't dump anything
    let compile_option2 = CompilerOption {
        output: CompilerOutputFormat::Path(bin_outputs[1].clone()),
        ..base_option.clone()
    };

    // 2. end to end compile (cached)
    let mut res = run(compile_option2);

    if res.has_error() {
        panic!("{}", res.concat_results());
    }

    let mut clean_count = 0;
    clean_irs(&dir_to_clean, &mut dummy_compiler_output, &mut clean_count);

    assert_eq!(clean_count, 1);

    let compile_option3 = CompilerOption {
        output: CompilerOutputFormat::HighIr,
        dump_hir_to: Some(hir_outputs[1].clone()),
        ..base_option.clone()
    };

    // 3. incremental compile code -> hir
    let mut res = run(compile_option3);

    if res.has_error() {
        panic!("{}", res.concat_results());
    }

    let compile_option4 = CompilerOption {
        output: CompilerOutputFormat::MidIr,
        dump_mir_to: Some(mir_outputs[1].clone()),
        ..base_option.clone()
    };

    // 4. incremental compile hir -> mir
    let mut res = run(compile_option4);

    if res.has_error() {
        panic!("{}", res.concat_results());
    }

    let compile_option5 = CompilerOption {
        output: CompilerOutputFormat::Path(bin_outputs[2].clone()),
        ..base_option.clone()
    };

    // 5. incremental compile mir -> bin  (bin is not implemented yet, but still the test makes sense)
    let mut res = run(compile_option5);

    if res.has_error() {
        panic!("{}", res.concat_results());
    }

    assert_same_output(&bin_outputs);
    assert_same_json(&hir_outputs);
    assert_same_json(&mir_outputs);

    remove_dir_all(&tmp_result_dir).unwrap();
    drop(lock);
}

// If this test panics, it leaves tmp files. You MUST run `nu clean.nu` to remove the tmp files.
//
// 1. compile the file directly from code to the end
// 2. compile the file using intermediate result from 1
// 3. make sure both return the same errors (must check the error message)
fn test_runner2(path: &str) {
    let lock = unsafe { LOCK.lock().unwrap() };

    let mut dummy_compiler_output = CompilerOutput::new();
    let tmp_file_name = join(".", &format!("__tmp_{:x}", rand::random::<u128>())).unwrap();
    let dir_to_clean = parent(path).unwrap();

    clean_irs(&dir_to_clean, &mut dummy_compiler_output, &mut 0);

    let compile_option = CompilerOption {
        input_file: Some(path.to_string()),
        do_not_compile_and_do_this: None,
        show_warnings: true,
        output: CompilerOutputFormat::Path(tmp_file_name.clone()),
        save_ir: true,
        dump_hir_to: None,
        dump_mir_to: None,
        ..CompilerOption::default()
    };

    // 1. end to end compile (clean)
    let mut res = run(compile_option.clone());
    let err1 = res.concat_results();

    // 2. end to end compile (cached)
    let mut res = run(compile_option);
    let err2 = res.concat_results();

    // there's no point in checking how many dirs it removes
    // because failed compilation might not leave irs
    clean_irs(&dir_to_clean, &mut dummy_compiler_output, &mut 0);

    if err1 != err2 {
        panic!("Inconsistent Errors:\n\n{err1}\n\n{err2}");
    }

    remove_file(&tmp_file_name).unwrap();
    drop(lock);
}

// when `--output` is None, `--stop-at` is None, and `--save-ir` is true,
// it check if it saves ir and creates no output files
fn test_runner3(path: &str) {
    let lock = unsafe { LOCK.lock().unwrap() };

    let mut dummy_compiler_output = CompilerOutput::new();
    let dir_to_clean = parent(path).unwrap();
    clean_irs(&dir_to_clean, &mut dummy_compiler_output, &mut 0);

    assert!(!exists(&join(&dir_to_clean, SAVE_IRS_AT).unwrap()));

    let compile_option = CompilerOption {
        input_file: Some(path.to_string()),
        do_not_compile_and_do_this: None,
        show_warnings: true,
        output: CompilerOutputFormat::None,
        save_ir: true,
        dump_hir_to: None,
        dump_mir_to: None,
        ..CompilerOption::default()
    };

    run(compile_option);
    assert!(exists(&join(&dir_to_clean, SAVE_IRS_AT).unwrap()));

    let mut clean_count = 0;
    clean_irs(&dir_to_clean, &mut dummy_compiler_output, &mut clean_count);

    assert_eq!(clean_count, 1);
    drop(lock);
}

fn assert_same_output(outputs: &Vec<Path>) {
    let bytes = outputs.iter().map(
        |path| read_bytes(path).unwrap()
    ).collect::<Vec<_>>();

    for (index, byte) in bytes.iter().enumerate() {
        if byte != &bytes[0] {
            panic!(
                "assertion_failures: contents of `{}` and `{}` are different",
                &outputs[0],
                &outputs[index],
            );
        }
    }
}

fn assert_same_json(files: &Vec<Path>) {
    let jsons = files.iter().map(
        |file| {
            let mut json = json::parse(&read_string(file).unwrap()).unwrap();

            // uids change over compilations
            remove_uids(&mut json);

            json
        }
    ).collect::<Vec<_>>();

    for (index, json) in jsons.iter().enumerate() {
        if json != &jsons[0] {
            panic!(
                "assertion_failures: contents of `{}` and `{}` are different\n------\n{}\n------\n{}",
                &files[0],
                &files[index],
                json.pretty(4),
                jsons[0].pretty(4),
            );
        }
    }
}

fn remove_uids(json: &mut JsonValue) {
    match json {
        JsonValue::Null
        | JsonValue::Short(_)
        | JsonValue::String(_)
        | JsonValue::Number(_)
        | JsonValue::Boolean(_) => {},
        JsonValue::Object(obj) => {
            for (k, v) in obj.iter_mut() {
                if k == "uid" {
                    *v = JsonValue::Null;
                }

                else {
                    remove_uids(v);
                }
            }
        },
        JsonValue::Array(arr) => {
            for element in arr.iter_mut() {
                remove_uids(element);
            }
        },
    }
}

macro_rules! stage_test {
    // sodigy files that successfully compile
    (steps, $test_name: ident, $path: expr) => {
        #[test]
        fn $test_name() {
            test_runner1($path);
            test_runner3($path);
        }
    };

    // sodigy files that leave errors or warnings
    (errors, $test_name: ident, $path: expr) => {
        #[test]
        fn $test_name() {
            test_runner2($path);
        }
    };
}

// TODO: use `join` functions

stage_test!(steps, stage_test1, "./samples/empty.sdg");
stage_test!(steps, stage_test2, "./samples/easy.sdg");
stage_test!(steps, stage_test3, "./samples/unused_names.sdg");
stage_test!(steps, stage_test4, "./samples/tests/main.sdg");

stage_test!(errors, errors_test1, "./samples/errors/parse_err1.sdg");
stage_test!(errors, errors_test2, "./samples/errors/name_err1.sdg");
stage_test!(errors, errors_test3, "./samples/errors/expr_err1.sdg");
stage_test!(errors, warnings_test1, "./samples/unused_names.sdg");
