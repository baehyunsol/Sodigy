use crate::{
    Assert,
    Bytecode,
    Const,
    ConstOrRegister,
    Executable,
    Func,
    Label,
    Let,
    Register,
};
use sodigy_error::{Error, Warning};
use sodigy_session::Session as SodigySession;
use sodigy_mir::{self as mir, Intrinsic};
use sodigy_span::Span;
use sodigy_string::{InternedString, unintern_string};
use std::collections::{HashMap, HashSet};

pub struct Session {
    pub intermediate_dir: String,
    pub func_param_count: usize,

    // for creating tmp labels
    pub label_counter: u32,

    // def_span to register map
    pub local_registers: HashMap<Span, Register>,

    // key: def_span of the built-in function (in sodigy std)
    pub intrinsics: HashMap<Span, Intrinsic>,

    pub funcs: Vec<Func>,
    pub lets: Vec<Let>,
    pub asserts: Vec<Assert>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_mir_session(mir_session: &mir::Session) -> Self {
        let intrinsics = Intrinsic::ALL_WITH_LANG_ITEM.iter().map(
            |(intrinsic, lang_item)| match mir_session.lang_items.get(*lang_item) {
                Some(span) => (*span, *intrinsic),
                None => panic!("lang item not found: {lang_item:?}"),
            }
        ).collect();

        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            func_param_count: 0,
            label_counter: 0,
            local_registers: HashMap::new(),
            intrinsics,
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

    pub fn pop_all_locals(&self, bytecodes: &mut Vec<Bytecode>) {
        for register in self.local_registers.values() {
            bytecodes.push(Bytecode::Pop(*register));
        }
    }

    pub fn enter_func(&mut self) {
        self.func_param_count = 0;
        self.label_counter = 0;
        self.local_registers = HashMap::new();
    }

    // Once you call this function, you can't do any operation on the bytecode.
    pub fn into_executable(
        &mut self,
        debug_info: bool,
    ) -> Executable {
        self.make_labels_static();
        let (bytecodes, interned_strings) = self.into_labeled_bytecodes();

        let mut debug_info_map = HashMap::new();
        let mut asserts = Vec::with_capacity(self.asserts.len());
        let mut anon_index = 0;
        self.asserts.sort_by_key(|assert| assert.keyword_span);

        for assert in self.asserts.iter() {
            let name = match assert.name {
                Some(name) => String::from_utf8_lossy(&unintern_string(name, &self.intermediate_dir).unwrap().unwrap()).to_string(),
                None => {
                    anon_index += 1;
                    format!("anonymous-{anon_index}")
                },
            };

            if debug_info {
                debug_info_map.insert(assert.label_id.unwrap(), format!("assertion {name}"));
            }

            asserts.push((assert.label_id.unwrap(), name));
        }

        if debug_info {
            for func in self.funcs.iter() {
                let func_name = unintern_string(func.name, &self.intermediate_dir).unwrap().unwrap();
                debug_info_map.insert(func.label_id.unwrap(), format!("fn {}", String::from_utf8_lossy(&func_name)));
            }

            for r#let in self.lets.iter() {
                let let_name = unintern_string(r#let.name, &self.intermediate_dir).unwrap().unwrap();
                debug_info_map.insert(r#let.label_id.unwrap(), format!("let {}", String::from_utf8_lossy(&let_name)));
            }
        }

        Executable {
            bytecodes,
            interned_strings: interned_strings.iter().map(
                |s| (*s, unintern_string(*s, &self.intermediate_dir).unwrap().unwrap())
            ).collect(),
            asserts,
            debug_info: if debug_info { Some(debug_info_map) } else { None },
        }
    }

    // Once you call this function, you can't do any operation on the bytecode.
    fn make_labels_static(&mut self) {
        // def_span -> number of local_labels
        let mut local_label_map = HashMap::new();

        for (def_span, bytecode) in self.funcs.iter().map(
            |func| (func.name_span, &func.bytecodes)
        ).chain(self.lets.iter().map(
            |r#let| (r#let.name_span, &r#let.bytecodes)
        )).chain(self.asserts.iter().map(
            |assert| (assert.keyword_span, &assert.bytecodes)
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
            make_labels_static(&mut func.bytecodes, &label_map, offset + 1);
            func.label_id = Some(offset);
        }

        for r#let in self.lets.iter_mut() {
            let offset = *label_map.get(&r#let.name_span).unwrap();

            // `offset` is for the `let` statement itself, so we have to add 1 to the offset
            make_labels_static(&mut r#let.bytecodes, &label_map, offset + 1);
            r#let.label_id = Some(offset);
        }

        for assert in self.asserts.iter_mut() {
            let offset = *label_map.get(&assert.keyword_span).unwrap();

            // `offset` is for the assertion itself, so we have to add 1 to the offset
            make_labels_static(&mut assert.bytecodes, &label_map, offset + 1);
            assert.label_id = Some(offset);
        }
    }

    // Make sure to run `make_labels_static` before calling this.
    fn into_labeled_bytecodes(&mut self) -> (HashMap<u32, Vec<Bytecode>>, HashSet<InternedString>) {
        let mut result = HashMap::new();
        let mut interned_strings = HashSet::new();
        let mut curr_label;

        for (label_id, bytecodes, def_span) in self.funcs.iter().map(
            |func| (func.label_id.unwrap(), &func.bytecodes, func.name_span)
        ).chain(self.lets.iter().map(
            |r#let| (r#let.label_id.unwrap(), &r#let.bytecodes, r#let.name_span)
        )).chain(self.asserts.iter().map(
            |assert| (assert.label_id.unwrap(), &assert.bytecodes, assert.keyword_span)
        )) {
            curr_label = label_id;
            let mut buffer: Vec<Bytecode> = vec![];

            for bytecode in bytecodes.iter() {
                match bytecode {
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
                    Bytecode::PushConst { value, .. } |
                    Bytecode::UpdateCompound { value: ConstOrRegister::Const(value), .. } => match value {
                        Const::String { s, .. } if !s.is_short_string() => {
                            interned_strings.insert(*s);
                        },
                        // If it's a long integer, it also has to be collected
                        // Const::Number(n) => todo!(),
                        _ => {},
                    },
                    _ => {
                        buffer.push(bytecode.clone());
                    },
                }
            }

            if !buffer.is_empty() {
                result.insert(curr_label, buffer);
            }
        }

        (result, interned_strings)
    }
}

fn count_local_labels(bytecodes: &[Bytecode]) -> u32 {
    let mut labels = HashSet::new();

    for bytecode in bytecodes.iter() {
        if let Bytecode::Label(Label::Local(n)) = bytecode {
            labels.insert(*n);
        }
    }

    labels.len() as u32
}

fn make_labels_static(bytecodes: &mut Vec<Bytecode>, map: &HashMap<Span, u32>, offset: u32) {
    for bytecode in bytecodes.iter_mut() {
        match bytecode {
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
