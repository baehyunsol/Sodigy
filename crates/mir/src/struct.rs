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
        let mut fields = vec![];
        let mut has_error = false;
        let struct_type = Type::Data {
            constructor_def_span: hir_struct.name_span.id().unwrap(),
            constructor_span: Span::None,
            args: if hir_struct.generics.is_empty() {
                None
            } else {
                Some(hir_struct.generics.iter().map(
                    |generic| Type::GenericParam {
                        def_span: generic.name_span.clone(),
                        span: Span::None,
                    }
                ).collect())
            },
            group_span: if hir_struct.generics.is_empty() { None } else { Some(Span::None) },
        };

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

        session.types.insert(hir_struct.name_span.clone(), struct_type);

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
