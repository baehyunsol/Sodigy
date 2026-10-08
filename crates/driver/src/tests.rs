use crate::{
    Backend,
    ColorWhen,
    Profile,
    StoreIrAt,
    ValidateTokenSpans,
    init_project,
    init_workers_and_compile,
};
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
        None,  // emit
        Some(Backend::Interpret),
        Profile::Test { std_assertions: false, filters: None },
        String::from("verify_built_ins/target/"),
        OptimizeLevel::None,
        &HashMap::new(),  // custom_error_levels
        false,  // dump-post-mir-log
        false,   // dump-timings
        0,  // graceful-shutdown
        8,  // jobs
        ColorWhen::Never,
        true,  // incremental-compilation
        ValidateTokenSpans::Never,
        true,   // verify-built-ins
        false,  // check-allocator
        false,  // debug-bytecode
        true,   // run
        true,   // quiet
    ).unwrap();

    remove_dir_all("verify_built_ins").unwrap();
}

