use crate::{Expr, Session};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    // pub r#type: Option<Type>,
    pub value: Expr,
}

impl Func {
    pub fn from_hir(hir_func: &hir::Func, session: &mut Session) -> Result<Func, ()> {
        Ok(Func {
            name: hir_func.name,
            name_span: hir_func.name_span,
            value: Expr::from_hir(&hir_func.value, session)?,
        })
    }
}
