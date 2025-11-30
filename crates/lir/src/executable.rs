use crate::{Bytecode, Label};
use std::collections::HashMap;

pub struct Executable {
    labels: HashMap<Label, Vec<Bytecode>>,
}
