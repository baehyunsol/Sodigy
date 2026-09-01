use crate::{
    Assert,
    Bytecode,
    ExprHash,
    Func,
    Let,
    Value,
};
use sodigy_error::FuncEffect;
use sodigy_span::{Span, SpanHash};
use std::collections::HashMap;

pub struct ObjectFile {
    pub data: Vec<(ExprHash, Value)>,
    pub code: Vec<CodeSection>,
    pub main_entry: Option<SpanHash>,
    pub asserts: Vec<SpanHash>,
}

// It can be a func, an assertion or a global let.
pub struct CodeSection {
    pub label: SpanHash,

    // debug info
    pub span: Option<Span>,

    pub kind: CodeKind,
    pub name: String,
    pub params: Option<usize>,
    pub effect: FuncEffect,
    pub code: Vec<Bytecode>,
}

pub enum CodeKind {
    Func,
    Let,
    Assert,
}

impl ObjectFile {
    pub fn new(
        lets: &mut Vec<Let>,
        funcs: &mut Vec<Func>,
        asserts: &mut Vec<Assert>,
        data_section: &mut HashMap<ExprHash, Value>,
    ) -> ObjectFile {
        let mut data: Vec<(ExprHash, Value)> = data_section.drain().collect();
        data.sort_by_key(|(h, _)| *h);

        ObjectFile {
            data,
            code: _,
            main_entry: None,  // TODO
            asserts: _,
        }
    }

    pub fn empty() -> ObjectFile {
        ObjectFile {
            data: vec![],
            code: vec![],
            main_entry: None,
            asserts: vec![],
        }
    }
}
