use crate::run;
use sodigy_clap::{CompilerOption, IrStage};
use sodigy_files::{read_bytes, remove_file};

// 1. code -> tokens -> hir
// 2. tokens (from saved ir) -> hir
// 3. hir (from saved ir)
// tests whether 1, 2 and 3 are identical

fn runner(path: &str) {
    // let's avoid name collisions with `rand::random`
    let file_name_prefix = format!("__{:x}", rand::random::<u128>());
    let tokens1 = format!("./{file_name_prefix}_tokens1.tokens");
    let hir1 = format!("./{file_name_prefix}_hir1.hir");
    let hir2 = format!("./{file_name_prefix}_hir2.hir");

    let base_comp_opt = CompilerOption {
        do_not_compile_and_print_this: None,
        output_path: None,
        save_ir: false,
        show_warnings: true,
        ..CompilerOption::default()
    };

    // TODO: `dump_tokens` dumps human-readable tokens, but the compiler reads tokens from endec-ed tokens...
    // it has to use `--save-ir` flag to dump tokens
    // TODO: more options for `--save-ir`

    // code -> tokens -> hir
    // saves `__XXX_tokens1.tokens`
    // saves `__XXX_hir1.hir`
    let opt1 = CompilerOption {
        input_files: vec![path.to_string()],
        output_format: IrStage::HighIr,
        dump_tokens: true,
        dump_tokens_to: Some(tokens1.clone()),
        dump_hir: true,
        dump_hir_to: Some(hir1.clone()),
        ..base_comp_opt.clone()
    };

    let errors1 = run(opt1).concat_results();

    let opt2 = CompilerOption {
        input_files: vec![tokens1.clone()],
        output_format: IrStage::HighIr,
        dump_tokens: false,
        dump_tokens_to: None,
        dump_hir: true,
        dump_hir_to: Some(hir2.clone()),
        ..base_comp_opt.clone()
    };

    let errors2 = run(opt2).concat_results();

    if errors1 != errors2 {
        panic!("Compilations are not consistent!\n\n{errors1}\n\n{errors2}");
    }

    let hir1_content = read_bytes(&hir1).unwrap();
    let hir2_content = read_bytes(&hir2).unwrap();

    remove_file(&tokens1).unwrap();
    remove_file(&hir1).unwrap();
    remove_file(&hir2).unwrap();
}

macro_rules! run_test {
    ($test_name: ident, $path: literal) => {
        #[test]
        fn $test_name() {
            runner($path);
        }
    }
}

// make sure that all the `.sdg` files have no compile-errors
run_test!(stage_dump_test1, "./samples/easy.sdg");
