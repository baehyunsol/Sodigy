use crate::Bytecode;

pub struct Executable {
    pub asserts: Vec<(/* name: */ String, /* label: */ usize)>,
    pub bytecodes: Vec<Bytecode>,
}

impl Executable {}
