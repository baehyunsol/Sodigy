use crate::{Expr, Session};
use sodigy_hir as hir;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Do {
    pub keyword_span: Span,
    pub value: Expr,
}

impl Do {
    pub fn from_hir(hir_do: &hir::Do, session: &mut Session) -> Result<Do, ()> {
        Ok(Do {
            keyword_span: hir_do.keyword_span.clone(),
            value: Expr::from_hir(&hir_do.value, session)?,
        })
    }
}
