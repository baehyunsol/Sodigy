use crate::run;
use sodigy_config::{CompilerOption, CompilerOutputFormat, DumpType};
use sodigy_files::{exists, get_all_sdg, join, remove_file};
use sodigy_endec::{DumpJson, Endec};
use sodigy_high_ir::HirSession;

// TODO
// 1. list all the files in `../../samples`, including sub-dirs
// 2. compile
// 3. if error, continue
// 4. save the hir session as a file
// 5. load the file
// 6. see if the 2 sessions are the same

#[test]
fn hir_endec_test() {
    let sdg_files = get_all_sdg("./samples/", true, "sdg").unwrap();

    for file in sdg_files.iter() {
        let dump_json_at = join(".", &format!("__tmp_{:x}.json", rand::random::<u64>())).unwrap();
        let dump_hir_at = join(".", &format!("__tmp_{:x}.hir", rand::random::<u64>())).unwrap();

        run(CompilerOption {
            do_not_compile_and_do_this: None,
            dump_hir_to: Some(dump_json_at.clone()),
            dump_mir_to: None,
            dump_type: DumpType::Json,
            input_path: Some(file.to_string()),
            or_pattern_expansion_limit: 512,
            output_format: CompilerOutputFormat::Hir,
            output_path: Some(dump_hir_at.clone()),
            raw_input: None,
            library_paths: None,
            show_warnings: true,
            verbosity: 0,
        });

        // if there's an error it would not write to the files

        if exists(&dump_hir_at) {
            let new_session = HirSession::load_from_file(&dump_hir_at).unwrap();
            let new_json = new_session.dump_json();

            // TODO: it has to compare new_json and old_json, but there's no function that parses a json string

            remove_file(&dump_hir_at).unwrap();
        }

        if exists(&dump_json_at) {
            remove_file(&dump_json_at).unwrap();
        }
    }
}
