use crate::run;
use sodigy_config::{
    CompilerOption,
    CompilerOutputFormat,
    DumpType,
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
    let iter_count = 4096;
    let tmp_file_name = join(".", &format!("__tmp_{:x}.sdg", rand::random::<u64>())).unwrap();
    let dummy_output_file = join(".", &format!("__tmp_{:x}.out", rand::random::<u64>())).unwrap();

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

    let mut res = run(CompilerOption {
        do_not_compile_and_do_this: None,
        input_path: Some(tmp_file_name.clone()),
        output_path: Some(dummy_output_file.clone()),
        output_format: CompilerOutputFormat::Mir,  // TODO: it has to be `CompilerOutputFormat::Binary`, but it's not implemented yet
        dump_hir_to: None,
        dump_mir_to: None,
        dump_type: DumpType::Json,
        show_warnings: true,
        verbosity: 0,
        or_pattern_expansion_limit: 32,
        raw_input: None,
        library_paths: None,
    });

    remove_file(&tmp_file_name).unwrap();
    remove_file(&dummy_output_file).unwrap();

    if res.has_error() {
        let (_, stderr) = res.concat_results();
        panic!("{stderr}");
    }
}
