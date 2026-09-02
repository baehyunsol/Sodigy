use crate::{Bytecode, ExprHash, Value};
use std::collections::HashMap;

pub struct Executable {
    pub data: HashMap<ExprHash, Value>,
    pub code: Vec<Bytecode>,
    pub main_entry: Option<usize>,
    pub asserts: Vec<(/* name: */ String, /* bytecode offset: */ usize)>,
}
