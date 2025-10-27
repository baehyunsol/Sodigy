use crate::Bytecode;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Executable {
    pub bytecodes: HashMap<u32, Vec<Bytecode>>,
    pub main_func: Option<u32>,

    // only top-level assertions (for running tests)
    pub asserts: Vec<(u32, String)>,

    // label_id to explanation map
    // only for functions, lets and top-level assertions
    pub debug_info: Option<HashMap<u32, String>>,
}
