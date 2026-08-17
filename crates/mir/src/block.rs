use crate::{Assert, Do, Expr, Let, Session};
use sodigy_hir as hir;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub asserts: Vec<Assert>,
    pub does: Vec<Do>,
    pub value: Box<Expr>,
}

impl Block {
    pub fn from_hir(hir_block: &hir::Block, session: &mut Session) -> Result<Block, ()> {
        let mut has_error = false;
        let mut lets = Vec::with_capacity(hir_block.lets.len());
        let mut asserts = Vec::with_capacity(hir_block.asserts.len());
        let mut does = Vec::with_capacity(hir_block.does.len());

        for r#let in hir_block.lets.iter() {
            match Let::from_hir(r#let, session) {
                Ok(l) => {
                    lets.push(l);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        for assert in hir_block.asserts.iter() {
            match Assert::from_hir(assert, session) {
                Ok(a) => {
                    asserts.push(a);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        for r#do in hir_block.does.iter() {
            match Do::from_hir(r#do, session) {
                Ok(d) => {
                    does.push(d);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        let value = match Expr::from_hir(&hir_block.value, session) {
            Ok(v) => Some(v),
            Err(_) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(Block {
                group_span: hir_block.group_span.clone(),
                lets,
                asserts,
                does,
                value: Box::new(value.unwrap()),
            })
        }
    }
}
