use crate::run;
use crate::stages::generate_path_for_ir;
use sodigy_clap::{CompilerOption, IrStage};
use sodigy_files::{
    create_dir,
    join,
    read_bytes,
    remove_dir_all,
};

// 1. code -> tokens -> hir
// 2. tokens (from saved ir) -> hir
// 3. hir (from saved ir)
// tests whether 1, 2 and 3 are identical

fn runner(path: &str) {
    // let's avoid name collisions with `rand::random`
    let tmp_dir_name = join(".", &format!("__tmp_{:x}", rand::random::<u64>())).unwrap();
    create_dir(&tmp_dir_name).unwrap();

    let dump_hir_to_1 = join(&tmp_dir_name, "hir1.hir").unwrap();
    let dump_hir_to_2 = join(&tmp_dir_name, "hir2.hir").unwrap();

    // We don't have to set `--ignore-saved-ir` in this case
    // -> input file changes each time!
    let base_comp_opt = CompilerOption {
        do_not_compile_and_do_this: None,
        output_path: None,
        save_ir: true,
        show_warnings: true,
        ..CompilerOption::default()
    };

    let opt1 = CompilerOption {
        input_file: Some(path.to_string()),
        output_format: IrStage::HighIr,
        dump_hir_to: Some(dump_hir_to_1.clone()),
        ..base_comp_opt.clone()
    };

    let errors1 = run(opt1, None).concat_results();
    let input2 = generate_path_for_ir(&path.to_string(), "tokens", false).unwrap();

    let opt2 = CompilerOption {
        input_file: Some(input2),
        output_format: IrStage::HighIr,
        dump_hir_to: Some(dump_hir_to_2.clone()),
        ..base_comp_opt.clone()
    };

    let errors2 = run(opt2, None).concat_results();

    let sep = "\n\n-------------------------\n\n";

    if errors1 != errors2 {
        panic!("Compilations are not consistent!{sep}{errors1}{sep}{errors2}");
    }

    let hir1_content = read_bytes(&dump_hir_to_1).unwrap();
    let hir2_content = read_bytes(&dump_hir_to_2).unwrap();

    if hir1_content != hir2_content {
        panic!(
            "Compilations are not consistent!{sep}{}{sep}{}",
            String::from_utf8_lossy(&hir1_content).to_string(),
            String::from_utf8_lossy(&hir2_content).to_string(),
        );
    }

    remove_dir_all(&tmp_dir_name).unwrap()
}

macro_rules! run_test {
    ($test_name: ident, $path: literal) => {
        #[test]
        fn $test_name() {
            runner($path);
        }
    }
}

// TODO: use join function
// make sure that all the `.sdg` files have no compile-errors
run_test!(stage_dump_test1, "./samples/easy.sdg");
run_test!(stage_dump_test2, "./samples/empty.sdg");
run_test!(stage_dump_test3, "./samples/unused_names.sdg");
