use crate::{Assert, Bytecode, Func, Label, Let, Register};
use sodigy_error::{Error, Warning};
use sodigy_session::Session as SodigySession;
use sodigy_mir as mir;
use sodigy_span::Span;
use std::collections::{HashMap, HashSet};

pub struct Session {
    pub intermediate_dir: String,
    pub func_arg_count: usize,

    // for creating tmp labels
    pub label_counter: u32,

    // def_span to register map
    pub local_registers: HashMap<Span, Register>,

    pub funcs: Vec<Func>,
    pub lets: Vec<Let>,
    pub asserts: Vec<Assert>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_mir_session(mir_session: &mir::Session) -> Self {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            func_arg_count: 0,
            label_counter: 0,
            local_registers: HashMap::new(),
            funcs: vec![],
            lets: vec![],
            asserts: vec![],
            errors: mir_session.errors.clone(),
            warnings: mir_session.warnings.clone(),
        }
    }

    pub fn get_tmp_label(&mut self) -> Label {
        self.label_counter += 1;
        Label::Local(self.label_counter - 1)
    }

    pub fn register_local_name(&mut self, def_span: Span) -> Register {
        let register = Register::Local(self.local_registers.len() as u32);
        self.local_registers.insert(def_span, register);
        register
    }

    pub fn pop_all_locals(&self, bytecode: &mut Vec<Bytecode>) {
        for register in self.local_registers.values() {
            bytecode.push(Bytecode::Pop(*register));
        }
    }

    pub fn enter_func(&mut self) {
        self.func_arg_count = 0;
        self.label_counter = 0;
        self.local_registers = HashMap::new();
    }

    // Once you call this function, you can't do any operation on the bytecode.
    pub fn make_labels_static(&mut self) {
        // def_span -> number of local_labels
        let mut local_label_map = HashMap::new();

        for (def_span, bytecode) in self.funcs.iter().map(
            |func| (func.name_span, &func.bytecode)
        ).chain(self.lets.iter().map(
            |r#let| (r#let.name_span, &r#let.bytecode)
        )).chain(self.asserts.iter().map(
            |assert| (assert.keyword_span, &assert.bytecode)
        )) {
            local_label_map.insert(def_span, count_local_labels(bytecode));
        }

        // def_span -> static id of label
        let mut label_map = HashMap::new();
        let mut cur = 0;

        // TODO: it iterates items in random order
        //       it'd be nice to give similar id to related items
        for (def_span, num_local_labels) in local_label_map.iter() {
            label_map.insert(*def_span, cur);
            cur += num_local_labels + 1;  // `+ 1` for itself
        }

        for func in self.funcs.iter_mut() {
            let offset = *label_map.get(&func.name_span).unwrap();

            // `offset` is for the function itself, so we have to add 1 to the offset
            make_labels_static(&mut func.bytecode, &label_map, offset + 1);
            func.label_id = Some(offset);
        }

        for r#let in self.lets.iter_mut() {
            let offset = *label_map.get(&r#let.name_span).unwrap();

            // `offset` is for the `let` statement itself, so we have to add 1 to the offset
            make_labels_static(&mut r#let.bytecode, &label_map, offset + 1);
            r#let.label_id = Some(offset);
        }

        for assert in self.asserts.iter_mut() {
            let offset = *label_map.get(&assert.keyword_span).unwrap();

            // `offset` is for the assertion itself, so we have to add 1 to the offset
            make_labels_static(&mut assert.bytecode, &label_map, offset + 1);
            assert.label_id = Some(offset);
        }
    }

    // Make sure to run `make_labels_static` before calling this.
    pub fn into_labeled_bytecode(&self) -> HashMap<u32, Vec<Bytecode>> {
        let mut result = HashMap::new();
        let mut curr_label;

        for (label_id, bytecode) in self.funcs.iter().map(
            |func| (func.label_id.unwrap(), &func.bytecode)
        ).chain(self.lets.iter().map(
            |r#let| (r#let.label_id.unwrap(), &r#let.bytecode)
        )).chain(self.asserts.iter().map(
            |assert| (assert.label_id.unwrap(), &assert.bytecode)
        )) {
            curr_label = label_id;
            let mut buffer: Vec<Bytecode> = vec![];

            for b in bytecode.iter() {
                match b {
                    Bytecode::Label(Label::Static(n)) => {
                        if !buffer.is_empty() {
                            if !buffer.last().unwrap().is_unconditional_jump() {
                                buffer.push(Bytecode::Goto(Label::Static(*n)));
                            }

                            result.insert(curr_label, buffer);
                            buffer = vec![];
                        }

                        curr_label = *n;
                    },
                    _ => {
                        buffer.push(*b);
                    },
                }
            }

            if !buffer.is_empty() {
                result.insert(curr_label, buffer);
            }
        }

        result
    }
}

fn count_local_labels(bytecode: &[Bytecode]) -> u32 {
    let mut labels = HashSet::new();

    for b in bytecode.iter() {
        if let Bytecode::Label(Label::Local(n)) = b {
            labels.insert(*n);
        }
    }

    labels.len() as u32
}

fn make_labels_static(bytecode: &mut Vec<Bytecode>, map: &HashMap<Span, u32>, offset: u32) {
    for b in bytecode.iter_mut() {
        match b {
            Bytecode::PushCallStack(label) |
            Bytecode::Goto(label) |
            Bytecode::Label(label) |
            Bytecode::JumpIf { label, .. } |
            Bytecode::JumpIfInit { label, .. } => {
                let old_label = *label;
                let new_label = match old_label {
                    Label::Local(n) => Label::Static(offset + n),
                    Label::Func(span) |
                    Label::Const(span) => Label::Static(*map.get(&span).unwrap()),
                    Label::Static(_) => old_label,
                };

                *label = new_label;
            },
            Bytecode::Push { .. } |
            Bytecode::PushConst { .. } |
            Bytecode::Pop(_) |
            Bytecode::PopCallStack |
            Bytecode::Intrinsic(_) |
            Bytecode::Return |
            Bytecode::UpdateCompound { .. } |
            Bytecode::ReadCompound { .. } => {},
        }
    }
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn get_warnings(&self) -> &[Warning] {
        &self.warnings
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
