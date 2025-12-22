use crate::{Expr, Session, Type};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub type_annot_span: Option<Span>,
    pub value: Expr,
}

impl Let {
    pub fn from_hir(hir_let: &hir::Let, session: &mut Session) -> Result<Let, ()> {
        let mut has_error = false;
        let type_annot_span = hir_let.type_annot.as_ref().map(|t| t.error_span_wide());

        match hir_let.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
            Some(Ok(type_annot)) => {
                session.types.insert(hir_let.name_span, type_annot);
            },
            Some(Err(())) => {
                has_error = true;
            },
            _ => {},
        }

        let value = match Expr::from_hir(&hir_let.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(Let {
                name: hir_let.name,
                name_span: hir_let.name_span,
                type_annot_span,
                value: value.unwrap(),
            })
        }
    }
}
