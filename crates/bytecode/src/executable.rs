use crate::Bytecode;

pub struct Executable {
    pub asserts: Vec<(/* name: */ String, /* bytecode offset: */ usize)>,
    pub bytecodes: Vec<Bytecode>,
}

impl Executable {}
