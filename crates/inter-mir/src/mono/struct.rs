use super::Monomorphization;
use crate::Session;
use sodigy_hir::{self as hir, StructShape};
use sodigy_mir::{Struct, Type};
use sodigy_span::Span;

impl Session {
    pub fn monomorphize_struct(&mut self, r#struct: &Struct, monomorphization: &Monomorphization) -> Struct {
        let new_struct_span = r#struct.name_span.monomorphize(monomorphization.id);
        let mut new_fields = Vec::with_capacity(r#struct.fields.len());

        for field in r#struct.fields.iter() {
            todo!()
        }

        Struct {
            name: r#struct.name,
            name_span: new_struct_span,
            generics: vec![],
            fields: new_fields,
        }
    }

    pub fn monomorphize_struct_shape(&mut self, struct_shape: &StructShape, monomorphization: &Monomorphization) -> StructShape {
        let new_fields = struct_shape.fields.iter().map(
            |field| hir::StructField {
                name: field.name,
                name_span: field.name_span.monomorphize(monomorphization.id),
                type_annot: {
                    // I don't remember how I handled this type...
                    // It's supposed to be None, but I'm not sure...
                    // TODO: Check this!!!
                    assert!(field.type_annot.is_none());

                    field.type_annot.clone()
                },
                default_value: {
                    // TODO: I'm not sure how I should handle this.
                    //       Does it make sense to monomorphize default values?
                    assert!(field.default_value.is_none());

                    field.default_value.clone()
                },
            }
        ).collect();

        StructShape {
            name: struct_shape.name,
            fields: new_fields,
            generics: vec![],
            generic_group_span: None,
            associated_funcs: struct_shape.associated_funcs.clone(),
            associated_lets: struct_shape.associated_lets.clone(),
        }
    }
}
