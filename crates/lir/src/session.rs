use crate::{
    Assert,
    Bytecode,
    DropType,
    Executable,
    Func,
    Label,
    Let,
    Memory,
};
use sodigy_error::{Error, Warning};
use sodigy_mir::{Intrinsic, Session as MirSession};
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_string::unintern_string;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,
    pub label_counter: u32,

    pub funcs: Vec<Func>,

    // only top-level ones
    pub asserts: Vec<Assert>,
    pub lets: Vec<Let>,

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
    pub lang_items: HashMap<String, Span>,
}

impl Session {
    pub fn from_mir(mir_session: &MirSession) -> Self {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            label_counter: 0,
            funcs: vec![],
            asserts: vec![],
            lets: vec![],
            local_values: HashMap::new(),
            drop_types: HashMap::new(),
            intrinsics: Intrinsic::ALL_WITH_LANG_ITEM.iter().map(
                |(intrinsic, lang_item)| (*mir_session.lang_items.get(*lang_item).unwrap(), *intrinsic)
            ).collect(),
            lang_items: mir_session.lang_items.clone(),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items.get(lang_item) {
            Some(s) => *s,
            None => panic!("TODO: lang_item `{lang_item}`"),
        }
    }

    pub fn get_local_label(&mut self) -> Label {
        self.label_counter += 1;
        Label::Local(self.label_counter - 1)
    }

    pub fn register_local_name(&mut self, name: Span) -> Memory {
        let i = self.local_values.len();
        self.local_values.insert(name, i);

        // TODO: we have to insert the actual drop type
        //       currently, it doesn't drop anything at all!
        self.drop_types.insert(name, DropType::Scalar);

        Memory::Stack(i)
    }

    pub fn drop_block(&mut self, names: &[Span]) {
        for name in names.iter() {
            let i = self.local_values.remove(name).unwrap();

            match self.drop_types.remove(name).unwrap() {
                DropType::Scalar => {},  // no drop
                _ => todo!(),
            }
        }
    }

    pub fn drop_all_locals(&mut self, bytecodes: &mut Vec<Bytecode>) {
        let local_values = self.drop_types.drain().map(
            |(name, drop_type)| (name, *self.local_values.get(&name).unwrap(), drop_type)
        ).collect::<Vec<_>>();

        for (name, index, drop_type) in local_values.iter() {
            match drop_type {
                DropType::Scalar => {},  // no drop
                _ => todo!(),
            }
        }

        self.local_values = HashMap::new();
        self.drop_types = HashMap::new();
    }

    pub fn into_executable(&self) -> Executable {
        let mut result = vec![];
        let mut label_map = HashMap::new();

        for (def_span, bytecodes) in self.asserts.iter().map(
            |assert| (assert.keyword_span, &assert.bytecodes)
        ).chain(
            self.lets.iter().map(
                |r#let| (r#let.name_span, &r#let.bytecodes)
            )
        ).chain(
            self.funcs.iter().map(
                |func| (func.name_span, &func.bytecodes)
            )
        ) {
            let mut curr_label = (def_span, Label::Global(def_span));
            let mut last_index = 0;

            // It does nothing in runtime, but we need this in order to flatten the labels.
            result.push(Bytecode::Label(Label::Global(def_span)));

            for (i, bytecode) in bytecodes.iter().enumerate() {
                match bytecode {
                    Bytecode::Label(label) => {
                        label_map.insert(curr_label, result.len());
                        result.extend(bytecodes[last_index..i].to_vec());
                        last_index = i + 1;
                        curr_label = (def_span, *label);
                    },
                    _ => {},
                }
            }

            label_map.insert(curr_label, result.len());
            result.extend(bytecodes[last_index..].to_vec());
        }

        let mut curr_item_span = Span::None;

        for bytecode in result.iter_mut() {
            match bytecode {
                Bytecode::Jump(label) |
                Bytecode::JumpIf { label, .. } |
                Bytecode::JumpIfUninit { label, .. } |
                Bytecode::PushCallStack(label) => {
                    let flattened_index = match *label {
                        Label::Local(ll) => label_map.get(&(curr_item_span, *label)).unwrap(),
                        Label::Global(s) => label_map.get(&(s, Label::Global(s))).unwrap(),
                        Label::Flatten(_) => unreachable!(),
                    };

                    *label = Label::Flatten(*flattened_index);
                },
                Bytecode::Label(Label::Global(def_span)) => {
                    curr_item_span = *def_span;
                },
                _ => {},
            }
        }

        Executable {
            asserts: self.asserts.iter().map(
                |assert| (
                    String::from_utf8_lossy(&unintern_string(assert.name, &self.intermediate_dir).unwrap().unwrap()).to_string(),
                    *label_map.get(&(assert.keyword_span, Label::Global(assert.keyword_span))).unwrap(),
                )
            ).collect(),
            bytecodes: result,
        }
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
