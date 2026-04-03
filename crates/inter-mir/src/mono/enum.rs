use super::Monomorphization;
use crate::Session;
use sodigy_hir::{self as hir, EnumShape};
use sodigy_mir::{Enum, EnumVariant, EnumVariantFields, Type};
use sodigy_span::Span;

impl Session {
    pub fn monomorphize_enum(&mut self, r#enum: &Enum, monomorphization: &Monomorphization) -> Enum {
        let new_enum_span = r#enum.name_span.monomorphize(monomorphization.id);
        let new_enum_type = Type::Data {
            constructor_def_span: new_enum_span.clone(),
            constructor_span: Span::None,
            args: None,
            group_span: None,
        };
        let mut new_variants = Vec::with_capacity(r#enum.variants.len());

        for variant in r#enum.variants.iter() {
            let new_variant_span = variant.name_span.monomorphize(monomorphization.id);
            let old_variant_type = self.types.get(&variant.name_span).unwrap();
            let mut new_variant_type: Type = old_variant_type.clone();

            for (generic_param, generic_arg) in monomorphization.generics.iter() {
                new_variant_type.substitute_generic_param(generic_param, generic_arg);
            }

            self.types.insert(new_variant_span.clone(), new_variant_type);
            let new_fields = match &variant.fields {
                EnumVariantFields::None | EnumVariantFields::Tuple(_) => variant.fields.clone(),
                EnumVariantFields::Struct(fields) => todo!(),  // monomorphize name_spans
            };

            new_variants.push(EnumVariant {
                name: variant.name,
                name_span: new_variant_span,
                fields: new_fields,
            });
        }

        Enum {
            name: r#enum.name,
            name_span: new_enum_span,
            generics: vec![],
            variants: new_variants,
        }
    }

    pub fn monomorphize_enum_shape(&mut self, enum_shape: &EnumShape, monomorphization: &Monomorphization) -> EnumShape {
        let new_variants: Vec<hir::EnumVariant> = enum_shape.variants.iter().map(
            |variant| hir::EnumVariant {
                name: variant.name,
                name_span: variant.name_span.monomorphize(monomorphization.id),
                fields: match &variant.fields {
                    hir::EnumVariantFields::None | hir::EnumVariantFields::Tuple(_) => variant.fields.clone(),
                    _ => todo!(),
                },
            }
        ).collect();

        EnumShape {
            name: enum_shape.name,
            variant_index: new_variants.iter().enumerate().map(
                |(index, variant)| (variant.name_span.clone(), index)
            ).collect(),
            variants: new_variants,

            // TODO: niche optimization
            representation: enum_shape.representation,

            generics: vec![],
            generic_group_span: None,
            associated_funcs: enum_shape.associated_funcs.clone(),
            associated_lets: enum_shape.associated_lets.clone(),
        }
    }
}
