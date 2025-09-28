use crate::{Expr, Func, Let, Session};
use sodigy_hir as hir;

pub struct Block {
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub value: Box<Option<Expr>>,
}

impl Block {
    pub fn from_hir(hir_block: &hir::Block, session: &mut Session) -> Result<Block, ()> {
        todo!()
    }
}
