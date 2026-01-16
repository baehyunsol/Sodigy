use crate::{Session, Type};
use sodigy_hir::{self as hir, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;

// `session.types` already has all the necessary information, so this
// struct only has names, which are required if you want to dump mir.
#[derive(Clone, Debug)]
pub struct Struct {
    pub name: InternedString,
    pub name_span: Span,
    pub fields: Vec<(InternedString, Span)>,
    pub generics: Vec<Generic>,
}

impl Struct {
    pub fn from_hir(hir_struct: &hir::Struct, session: &mut Session) -> Result<Struct, ()> {
        let mut has_error = false;
        let mut fields = vec![];

        for generic in hir_struct.generics.iter() {
            session.generic_def_span_rev.insert(generic.name_span, hir_struct.name_span);
        }

        for field in hir_struct.fields.iter() {
            match field.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
                Some(Ok(type_annot)) => {
                    session.types.insert(field.name_span, type_annot);
                },
                None => {
                    session.types.insert(
                        field.name_span,
                        Type::Var {
                            def_span: field.name_span,
                            is_return: false,
                        },
                    );
                },
                Some(Err(())) => {
                    has_error = true;
                    continue;
                },
            }

            fields.push((field.name, field.name_span));
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Struct {
                name: hir_struct.name,
                name_span: hir_struct.name_span,
                fields,
                generics: hir_struct.generics.clone(),
            })
        }
    }
}
