use crate::{Assert, Bytecode, DropType, Func, Label, Let, Memory};
use sodigy_error::{Error, Warning};
use sodigy_mir::{Intrinsic, Session as MirSession};
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
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
