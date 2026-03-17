#![no_main]

use libfuzzer_sys::fuzz_target;
use sodigy_fuzz::runner;

fuzz_target!(|data: &[u8]| runner(data, "cnr"));
