use crate::{CompilerOutput, clean_irs, run};
use sodigy_clap::{CompilerOption, CompilerOutputFormat};
use sodigy_files::{
    create_dir,
    join,
    parent,
    read_bytes,
    remove_dir_all,
};
use std::sync::Mutex;

static mut LOCK: Mutex<()> = Mutex::new(());

type Path = String;

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

    assert_same_output(&bin_outputs, /* is_json */ false);
    assert_same_output(&hir_outputs, /* is_json */ true);
    assert_same_output(&mir_outputs, /* is_json */ true);

    remove_dir_all(&tmp_result_dir).unwrap();
    drop(lock);
}

// TODO: another test
// a file that has a parse error
// 1. compile the file directly to mir
// 2. compile the file from code to hir, then to mir
// 3. make sure both return the same errors (must check the error message)
fn test_runner2(path: &str) {}

fn assert_same_output(outputs: &Vec<Path>, is_json: bool) {
    let bytes = outputs.iter().map(
        |path| read_bytes(path).unwrap()
    ).collect::<Vec<_>>();

    // TODO: do not compare uids -> uids change over compilations
    for (index, byte) in bytes.iter().enumerate() {
        if byte != &bytes[0] {
            panic!(
                "assertion_failures: contents of `{}` and `{}` are different{}",
                &outputs[0],
                &outputs[index],
                if is_json {
                    String::new()  // TODO: dump the content of json
                } else {
                    String::new()
                },
            );
        }
    }
}

// TODO: another test
// set `--output` to None, `--stop-at` to None, and `--save-ir` to true
// check if it saves ir and creates no output files

macro_rules! stage_test {
    (steps, $test_name: ident, $path: expr) => {
        #[test]
        fn $test_name() {
            test_runner1($path);
        }
    };

    (errors, $test_name: ident, $path: expr) => {
        #[test]
        fn $test_name() {
            test_runner2($path);
        }
    };
}

// TODO: use `join` functions

// make sure that these files have no compile errors
stage_test!(steps, stage_test1, "./samples/empty.sdg");
stage_test!(steps, stage_test2, "./samples/easy.sdg");
stage_test!(steps, stage_test3, "./samples/unused_names.sdg");

// make sure that these files have compile errors
stage_test!(errors, errors_test1, "./samples/errors/parse_err1.sdg");
