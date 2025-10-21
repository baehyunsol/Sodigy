use crate::{Expr, Session, Type};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub value: Expr,
}

impl Let {
    pub fn from_hir(hir_let: &hir::Let, session: &mut Session) -> Result<Let, ()> {
        let mut has_error = false;
        match hir_let.r#type.as_ref().map(|r#type| Type::from_hir(r#type, session)) {
            Some(Ok(r#type)) => {
                session.types.insert(hir_let.name_span, r#type);
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
                value: value.unwrap(),
            })
        }
    }
}
