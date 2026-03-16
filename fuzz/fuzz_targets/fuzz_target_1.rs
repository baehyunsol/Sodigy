#![no_main]

use libfuzzer_sys::fuzz_target;

use sodigy_driver::{
    Backend,
    ColorWhen,
    OptimizeLevel,
    Profile,
    StoreIrAt,
    init_workers_and_compile,
};
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    // TODO: init ir dir
    // TODO: write `data` to `src/lib.sdg`
    init_workers_and_compile(
        StoreIrAt::IntermediateDir,
        Backend::Bytecode,
        String::from("__fuzz"),
        OptimizeLevel::None,
        true,
        &HashMap::new(),
        true,
        0,
        8,
        ColorWhen::Never,
        true,
        Some(Profile::Test),
    ).unwrap();
});
