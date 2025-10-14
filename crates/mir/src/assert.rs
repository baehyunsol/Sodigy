use crate::{Expr, Session};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Assert {
    pub keyword_span: Span,
    pub error_message: InternedString,
    pub value: Expr,
}

impl Assert {
    pub fn from_hir(hir_assert: &hir::Assert, session: &mut Session) -> Result<Assert, ()> {
        Ok(Assert {
            keyword_span: hir_assert.keyword_span,
            error_message: InternedString::empty(),
            value: Expr::from_hir(&hir_assert.value, session)?,
        })
    }
}
