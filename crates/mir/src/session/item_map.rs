use crate::{
    Assert,
    Enum,
    Func,
    Let,
    Session,
    Struct,
};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct ItemMap {
    pub lets: HashMap<Span, Let>,
    pub funcs: HashMap<Span, Func>,
    pub enums: HashMap<Span, Enum>,
    pub structs: HashMap<Span, Struct>,
    pub asserts: HashMap<Span, Assert>,
}

impl Session {
    pub fn get_item_map(&mut self) -> ItemMap {
        ItemMap {
            lets: self.lets.drain(..).map(|r#let| (r#let.name_span, r#let)).collect(),
            funcs: self.funcs.drain(..).map(|func| (func.name_span, func)).collect(),
            enums: self.enums.drain(..).map(|r#enum| (r#enum.name_span, r#enum)).collect(),
            structs: self.structs.drain(..).map(|r#struct| (r#struct.name_span, r#struct)).collect(),
            asserts: self.asserts.drain(..).map(|assert| (assert.keyword_span, assert)).collect(),
        }
    }

    pub fn update_items(&mut self, items: &ItemMap) {
        let mut lets = Vec::with_capacity(self.lets.len());
        let mut funcs = Vec::with_capacity(self.funcs.len());
        let mut enums = Vec::with_capacity(self.enums.len());
        let mut structs = Vec::with_capacity(self.structs.len());
        let mut asserts = Vec::with_capacity(self.asserts.len());

        for r#let in self.lets.iter() {
            lets.push(items.lets.get(&r#let.name_span).unwrap().clone());
        }

        for func in self.funcs.iter() {
            funcs.push(items.funcs.get(&func.name_span).unwrap().clone());
        }

        for r#enum in self.enums.iter() {
            enums.push(items.enums.get(&r#enum.name_span).unwrap().clone());
        }

        for r#struct in self.structs.iter() {
            structs.push(items.structs.get(&r#struct.name_span).unwrap().clone());
        }

        for assert in self.asserts.iter() {
            asserts.push(items.asserts.get(&assert.keyword_span).unwrap().clone());
        }

        self.lets = lets;
        self.funcs = funcs;
        self.enums = enums;
        self.structs = structs;
        self.asserts = asserts;
    }
}
