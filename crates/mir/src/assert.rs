use crate::{Expr, Session};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Assert {
    pub name: Option<InternedString>,
    pub keyword_span: Span,
    pub error_message: InternedString,
    pub value: Expr,
    pub always: bool,
}

impl Assert {
    pub fn from_hir(hir_assert: &hir::Assert, session: &mut Session) -> Result<Assert, ()> {
        Ok(Assert {
            name: hir_assert.name,
            keyword_span: hir_assert.keyword_span,
            error_message: InternedString::empty(),
            value: Expr::from_hir(&hir_assert.value, session)?,
            always: hir_assert.always,
        })
    }
}
