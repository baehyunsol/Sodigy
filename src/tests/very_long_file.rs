use crate::run;
use sodigy_clap::{
    CompilerOption,
    IrStage,
};
use sodigy_files::{
    join,
    remove_file,
    write_string,
    WriteMode,
};

fn random_string(len: usize) -> String {
    (0..len).map(
        |_| ((rand::random::<u8>() & 15) + b'a') as char
    ).collect::<String>()
}

#[test]
fn very_long_file() {
    return;  // TODO: let's not do this for now

    let iter_count = 8192;
    let tmp_file_name = join(".", &format!("__tmp_{:x}.sdg", rand::random::<u64>())).unwrap();

    write_string(&tmp_file_name, "# very long file test\n", WriteMode::AlwaysCreate).unwrap();
    write_string(&tmp_file_name, "let numbers = [\n", WriteMode::AlwaysAppend).unwrap();

    for _ in 0..iter_count {
        write_string(
            &tmp_file_name,
            &format!(
                "    # {}\n", random_string(64),
            ),
            WriteMode::AlwaysAppend,
        ).unwrap();
        write_string(
            &tmp_file_name,
            &format!(
                "    {:#x}, {:#x}, {:#x}, {:#x},\n",
                rand::random::<u128>(),
                rand::random::<u128>(),
                rand::random::<u128>(),
                rand::random::<u128>(),
            ),
            WriteMode::AlwaysAppend,
        ).unwrap();
    }

    write_string(&tmp_file_name, "];\n\nlet strings = [\n", WriteMode::AlwaysAppend).unwrap();

    for _ in 0..iter_count {
        write_string(
            &tmp_file_name,
            &format!(
                "    # {}\n", random_string(64),
            ),
            WriteMode::AlwaysAppend,
        ).unwrap();
        write_string(
            &tmp_file_name,
            &format!("    \"{}\",\n", random_string(rand::random::<usize>() & 63 | 64)),
            WriteMode::AlwaysAppend,
        ).unwrap();
    }

    write_string(&tmp_file_name, "];", WriteMode::AlwaysAppend).unwrap();

    run(
        CompilerOption {
            do_not_compile_and_do_this: None,
            input_file: Some(tmp_file_name.clone()),
            output_path: None,
            output_format: IrStage::HighIr,
            show_warnings: true,
            save_ir: true,
            dump_tokens: false,
            dump_tokens_to: None,
            dump_hir: false,
            dump_hir_to: None,
            verbosity: 0,
            raw_input: None,
        },
        None,
    );

    remove_file(&tmp_file_name).unwrap();
}
