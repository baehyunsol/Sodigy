use crate::{Bytecode, DropType, Label, Memory};
use sodigy_error::{Error, Warning};
use sodigy_mir::Intrinsic;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,
    pub label_counter: u32,
    pub func_param_count: usize,

    // `Span` is the name span of func param or local value (`let`).
    // It'll give you the stack offset.
    pub local_values: HashMap<Span, usize>,

    // When you `register_local_name`, the session remembers
    // how it should drop the local value.
    // When you `drop_all_locals`, the session drops all the
    // locals in this map.
    pub drop_types: HashMap<Span, DropType>,

    // key: def_span of the built-in function (in sodigy std)
    pub intrinsics: HashMap<Span, Intrinsic>,
}

impl Session {
    pub fn get_local_label(&mut self) -> Label {
        self.label_counter += 1;
        Label::Local(self.label_counter - 1)
    }

    pub fn register_local_name(&mut self, name: Span) -> Memory {
        todo!()
    }

    pub fn drop_block(&mut self, names: &[Span]) {
        todo!()
    }

    pub fn drop_all_locals(&mut self, bytecodes: &mut Vec<Bytecode>) {
        todo!()
    }
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &[]
    }

    fn get_warnings(&self) -> &[Warning] {
        &[]
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
