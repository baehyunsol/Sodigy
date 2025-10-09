use crate::{Expr, Let, Session};
use sodigy_hir as hir;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub value: Box<Expr>,
}

impl Block {
    pub fn from_hir(hir_block: &hir::Block, session: &mut Session) -> Result<Block, ()> {
        let mut lets = vec![];
        let mut has_error = false;

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
                group_span: hir_block.group_span,
                lets: lets,
                value: Box::new(value.unwrap()),
            })
        }
    }
}
