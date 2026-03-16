#![no_main]

use libfuzzer_sys::fuzz_target;

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
    remove_dir_all,
    write_bytes,
};
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    if exists("sodigy-fuzz-test") {
        remove_dir_all("sodigy-fuzz-test").unwrap();
    }

    init_project("sodigy-fuzz-test").unwrap();
    write_bytes(
        "sodigy-fuzz-test/src/lib.sdg",
        data,
        WriteMode::CreateOrTruncate,
    ).unwrap();

    // TODO: don't make it dump anything to stdout/stderr
    match init_workers_and_compile(
        String::from("sodigy-fuzz-test/src/"),
        StoreIrAt::IntermediateDir,
        Backend::Bytecode,
        String::from("sodigy-fuzz-test/target/"),
        OptimizeLevel::None,
        true,
        &HashMap::new(),
        true,
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
});
