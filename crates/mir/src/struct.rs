use crate::{Session, Type};
use sodigy_hir::{self as hir, Generic};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_span::Span;
use sodigy_string::InternedString;

// `session.types` already has all the necessary information, so this
// struct only has names, which are required if you want to dump mir.
#[derive(Clone, Debug)]
pub struct Struct {
    pub name: InternedString,
    pub name_span: Span,
    pub fields: Vec<StructField>,
    pub generics: Vec<Generic>,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub name: InternedString,
    pub name_span: Span,
    pub default_value: Option<IdentWithOrigin>,
}

impl Struct {
    pub fn from_hir(hir_struct: &hir::Struct, session: &mut Session) -> Result<Struct, ()> {
        let mut has_error = false;
        let mut fields = vec![];

        for field in hir_struct.fields.iter() {
            match field.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
                Some(Ok(type_annot)) => {
                    session.types.insert(field.name_span.clone(), type_annot);
                },
                None => {
                    session.types.insert(
                        field.name_span.clone(),
                        Type::Var {
                            def_span: field.name_span.clone(),
                            is_return: false,
                        },
                    );
                },
                Some(Err(())) => {
                    has_error = true;
                    continue;
                },
            }

            fields.push(StructField {
                name: field.name,
                name_span: field.name_span.clone(),
                default_value: field.default_value.clone(),
            });
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Struct {
                name: hir_struct.name,
                name_span: hir_struct.name_span.clone(),
                fields,
                generics: hir_struct.generics.clone(),
            })
        }
    }
}
