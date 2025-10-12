use crate::{Bytecode, Label, Register};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Session {
    pub func_arg_count: usize,

    // for creating tmp labels
    pub label_counter: u32,

    // def_span to register map
    pub local_registers: HashMap<Span, Register>,

    pub funcs: HashMap<InternedString, Vec<Bytecode>>,
    pub lets: HashMap<InternedString, Vec<Bytecode>>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            func_arg_count: 0,
            label_counter: 0,
            local_registers: HashMap::new(),
            funcs: HashMap::new(),
            lets: HashMap::new(),
        }
    }

    pub fn get_tmp_label(&mut self) -> Label {
        self.label_counter += 1;
        Label::Local(self.label_counter)
    }

    pub fn register_local_name(&mut self, def_span: Span) -> Register {
        let register = Register::Local(self.local_registers.len() as u32);
        self.local_registers.insert(def_span, register);
        register
    }

    pub fn enter_func(&mut self) {
        self.func_arg_count = 0;
        self.label_counter = 0;
        self.local_registers = HashMap::new();
    }
}
