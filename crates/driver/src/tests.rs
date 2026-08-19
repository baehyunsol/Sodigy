use crate::{
    ColorWhen,
    Profile,
    StoreIrAt,
    ValidateTokenSpans,
    init_project,
    init_workers_and_compile,
};
use sodigy_code_gen::Backend;
use sodigy_optimize::OptimizeLevel;
use sodigy_fs_api::{exists, remove_dir_all};
use std::collections::HashMap;

#[test]
fn verify_built_ins() {
    if exists("verify_built_ins") {
        remove_dir_all("verify_built_ins").unwrap();
    }

    init_project("verify_built_ins").unwrap();
    init_workers_and_compile(
        String::from("verify_built_ins/src"),
        StoreIrAt::IntermediateDir,
        Backend::Bytecode,
        String::from("verify_built_ins/target/"),
        OptimizeLevel::None,
        &HashMap::new(),
        false,  // emit-irs
        false,  // dump-post-mir-log
        false,   // dump-timings
        false,  // dump-bytecodes
        0,  // graceful-shutdown
        8,  // jobs
        ColorWhen::Never,
        true,  // incremental-compilation
        ValidateTokenSpans::Never,
        true,  // verify-built-ins
        Some(Profile::Test),
        true,  // quiet
    ).unwrap();

    remove_dir_all("verify_built_ins").unwrap();
}
