use crate::{Assert, ExprHash, Func, Let};

pub struct ObjectFile {
    pub data: Vec<(ExprHash, Value)>,
    pub codes: Vec<Code>,
    pub main_entry: Option<Label>,
    pub asserts: Vec<Label>,
}

// It can be a func, an assertion or a global let.
// FIXME: I don't like its name.
pub struct Code {
    pub label: Label,
    pub name: String,
    pub kind: CodeKind,
    pub effect: FuncEffect,
    pub code: Vec<Bytecode>,
}

pub enum CodeKind {
    Func,
    Let,
    Assert,
}

impl ObjectFile {
    pub fn new(lets: &mut Vec<Let>, funcs: &mut Vec<Func>, asserts: &mut Vec<Assert>) -> ObjectFile {}

    pub fn empty() -> ObjectFile {
        ObjectFile {
            data: vec![],
            codes: vec![],
            main_entry: None,
            asserts: vec![],
        }
    }
}
