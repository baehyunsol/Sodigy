use crate::{Expr, Session};
use sodigy_hir as hir;

#[derive(Clone, Debug)]
pub struct Assert {
    pub value: Expr,
}

impl Assert {
    pub fn from_hir(hir_assert: &hir::Assert, session: &mut Session) -> Result<Assert, ()> {
        Ok(Assert {
            value: Expr::from_hir(&hir_assert.value, session)?,
        })
    }
}
