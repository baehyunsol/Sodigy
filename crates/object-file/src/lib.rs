use sodigy_bytecode::{
    Bytecode,
    ExprHash,
    GlobalLabel,
    LocalLabel,
    Memory,
    SSA,
    Value,
};
use sodigy_error::FuncEffect;
use sodigy_span::Span;
use std::collections::HashMap;

mod assert;
mod basic_block;
mod dump;
mod endec;
mod link;
mod parse;
mod profile;
mod ref_count;
mod session;

pub use assert::{Assert, AssertionFilter};
pub use basic_block::{BasicBlock, Terminator, to_basic_blocks};
pub use dump::Highlight;
pub use link::link;
pub use parse::{ParseError, parse};
pub use profile::Profile;
pub use ref_count::insert_ref_count;
pub use session::Session;

pub struct ObjectFile {
    pub data: HashMap<ExprHash, Value>,
    pub code: HashMap<GlobalLabel, Code>,
    pub entry: Entry,
}

// It can be a func, an assertion or a global let.
#[derive(Clone, Debug)]
pub struct Code {
    pub label: GlobalLabel,

    // debug info
    pub span: Option<Span>,

    pub kind: CodeKind,
    pub name: String,
    pub params: Option<usize>,
    pub effect: FuncEffect,
    pub basic_blocks: HashMap<LocalLabel, BasicBlock>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CodeKind {
    Func,
    Let,
    Assert,
}

#[derive(Clone, Debug)]
pub enum Entry {
    Main(GlobalLabel),
    Asserts(Vec<Assert>),

    // The compiler will turn this into an error later.
    NoEntry,
}

