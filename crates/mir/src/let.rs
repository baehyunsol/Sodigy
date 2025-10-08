use crate::{Expr, Session};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    // pub r#type: Option<Type>,
    pub value: Expr,
}

impl Let {
    pub fn from_hir(hir_let: &hir::Let, session: &mut Session) -> Result<Let, ()> {
        match Expr::from_hir(&hir_let.value, session) {
            Ok(value) => Ok(Let {
                name: hir_let.name,
                name_span: hir_let.name_span,
                value,
            }),
            Err(()) => Err(()),
        }
    }
}
