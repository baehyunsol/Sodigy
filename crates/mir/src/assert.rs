use crate::{Expr, Session};
use sodigy_hir as hir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Assert {
    // TODO: keep it `Option<InternedString>` and make the compiler name it
    //       vs
    //       make it `InternedString` and give it a name with some hashes
    pub name: Option<InternedString>,
    pub keyword_span: Span,
    pub always: bool,
    pub note: Option<Expr>,
    pub note_decorator_span: Option<Span>,
    pub value: Expr,
}

impl Assert {
    pub fn from_hir(hir_assert: &hir::Assert, session: &mut Session) -> Result<Assert, ()> {
        let mut has_error = false;

        let note = match hir_assert.note.as_ref().map(|note| Expr::from_hir(note, session)) {
            Some(Ok(note)) => Some(note),
            Some(Err(())) => {
                has_error = true;
                None
            },
            None => None,
        };

        let value = match Expr::from_hir(&hir_assert.value, session) {
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
            Ok(Assert {
                name: hir_assert.name,
                keyword_span: hir_assert.keyword_span,
                always: hir_assert.always,
                note,
                note_decorator_span: hir_assert.note_decorator_span,
                value: value.unwrap(),
            })
        }
    }
}
