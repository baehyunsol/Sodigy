use sodigy_driver::{
    Backend,
    ColorWhen,
    Error,
    OptimizeLevel,
    Profile,
    StoreIrAt,
    init_project,
    init_workers_and_compile,
};
use sodigy_fs_api::{
    WriteMode,
    exists,
    join,
    remove_dir_all,
    write_bytes,
};
use std::collections::HashMap;

// TODO: accept multiple modules
pub fn runner(data: &[u8], target: &str) {
    let target_dir = format!("sdg-src-{target}");

    if exists(&target_dir) {
        remove_dir_all(&target_dir).unwrap();
    }

    init_project(&target_dir).unwrap();
    write_bytes(
        &join(&target_dir, "src/lib.sdg").unwrap(),
        data,
        WriteMode::CreateOrTruncate,
    ).unwrap();

    match init_workers_and_compile(
        join(&target_dir, "src/").unwrap(),
        StoreIrAt::IntermediateDir,
        Backend::Bytecode,
        join(&target_dir, "target/").unwrap(),
        OptimizeLevel::None,
        true,
        &HashMap::new(),

        // If it's true, I can find bugs in ir dumps.
        // If it's false, the fuzzer's evolution algorithm will become more efficient.
        false,  // emit-irs

        0,
        8,
        ColorWhen::Never,
        true,
        Some(Profile::Test),
        true,
    ) {
        Ok(_) => {},
        Err(Error::CompileError) => {},  // it's okay
        Err(e) => panic!("{e:?}"),
    }
}
