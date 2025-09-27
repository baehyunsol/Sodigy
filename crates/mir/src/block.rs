use crate::{Expr, Func, Let, RefCount};
use sodigy_hir as hir;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Block {
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,

    // def_span to ref_count map, for EVERY Identifier in value
    pub value_name_count: HashMap<Span, RefCount>,
    pub value: Box<Option<Expr>>,
}

impl Block {
    pub fn from_hir(hir_block: &hir::Block) -> Result<Block, ()> {
        todo!()
    }
}
