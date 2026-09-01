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
use sodigy_string::unintern_string;
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
        intermediate_dir: &str,
    ) -> ObjectFile {
        let mut code = Vec::with_capacity(lets.len() + funcs.len() + asserts.len());
        let mut data: Vec<(ExprHash, Value)> = data_section.drain().collect();
        let mut assert_labels = Vec::with_capacity(asserts.len());
        data.sort_by_key(|(h, _)| *h);

        for mut func in funcs.drain(..) {
            code.push(CodeSection {
                label: func.name_span.hash(),
                span: Some(func.name_span.clone()),
                kind: CodeKind::Func,
                name: String::from_utf8_lossy(&unintern_string(func.name, intermediate_dir).unwrap().unwrap()).to_string(),
                params: Some(func.params),
                effect: func.effect.clone(),
                code: std::mem::take(&mut func.bytecodes),
            });
        }

        for mut r#let in lets.drain(..) {
            code.push(CodeSection {
                label: r#let.name_span.hash(),
                span: Some(r#let.name_span.clone()),
                kind: CodeKind::Let,
                name: String::from_utf8_lossy(&unintern_string(r#let.name, intermediate_dir).unwrap().unwrap()).to_string(),
                params: None,
                effect: FuncEffect::Fn,
                code: std::mem::take(&mut r#let.bytecodes),
            });
        }

        for mut assert in asserts.drain(..) {
            let label = assert.keyword_span.hash();
            assert_labels.push(label);
            code.push(CodeSection {
                label,
                span: Some(assert.keyword_span.clone()),
                kind: CodeKind::Assert,
                name: String::from_utf8_lossy(&unintern_string(assert.name, intermediate_dir).unwrap().unwrap()).to_string(),
                params: None,
                effect: FuncEffect::Fn,
                code: std::mem::take(&mut assert.bytecodes),
            });
        }

        ObjectFile {
            data,
            code,
            main_entry: None,  // TODO
            asserts: assert_labels,
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
