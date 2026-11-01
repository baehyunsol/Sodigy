use sodigy_bytecode::{
    Bytecode,
    ExprHash,
    GlobalLabel,
    LocalLabel,
    SSA,
    Value,
};
use sodigy_endec::Endec;
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
pub use parse::parse;
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
    Fn,
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

// It tells whether a `Vec<u8>` is a human-readable object-file format,
// or an endec-ed format.
pub fn is_binary_object_file(bytes: &[u8]) -> bool {
    // TODO: It depends on the implementation detail of the `Endec` trait
    //       and I don't like that.
    match usize::decode_impl(bytes, 0) {
        // I don't think an object file can be this big.
        // Also, if it's too big, rust's memory allocator will panic and
        // we can't handle that.
        Ok((0x3fff_ffff.., _)) => false,
        Err(_) => false,
        Ok((buffer_len, cursor)) => buffer_len > 8 && bytes[cursor..].starts_with(b"\0\0SDGOBJ"),
    }
}

