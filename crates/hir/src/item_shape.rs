use crate::{
    AssociatedFunc,
    AssociatedItem,
    AssociatedItemKind,
    EnumShape,
    StructShape,
};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub enum ItemShapeMut<'s> {
    Struct(&'s mut StructShape),
    Enum(&'s mut EnumShape),
}

pub enum ItemShape<'s> {
    Struct(&'s StructShape),
    Enum(&'s EnumShape),
}

macro_rules! item_shape_impl {
    ($type_name:ident) => {
        impl $type_name<'_> {
            // I tried returning `Box<dyn Iterator<Item=AssociatedItem>>`, but there was a
            // lifetime issue. I couldn't figure out how to fix, so I just collect the iterator.
            pub fn existing_associations(&self) -> Vec<AssociatedItem> {
                match self {
                    $type_name::Struct(s) => s.fields.iter().map(
                        |field| AssociatedItem {
                            kind: AssociatedItemKind::Field,
                            name: field.name,
                            name_span: field.name_span.clone(),
                            ..AssociatedItem::default()
                        }
                    ).chain(
                        s.associated_funcs.iter().map(
                            |(name, AssociatedFunc { is_pure, params, name_spans, .. })| AssociatedItem {
                                kind: AssociatedItemKind::Func,
                                name: *name,
                                name_span: name_spans[0].clone(),
                                is_pure: Some(*is_pure),
                                params: Some(*params),
                                ..AssociatedItem::default()
                            }
                        )
                    ).chain(
                        s.associated_lets.iter().map(
                            |(name, name_span)| AssociatedItem {
                                kind: AssociatedItemKind::Let,
                                name: *name,
                                name_span: name_span.clone(),
                                ..AssociatedItem::default()
                            }
                        )
                    ).collect(),
                    $type_name::Enum(e) => e.variants.iter().map(
                        |variant| AssociatedItem {
                            kind: AssociatedItemKind::Variant,
                            name: variant.name,
                            name_span: variant.name_span.clone(),
                            ..AssociatedItem::default()
                        }
                    ).chain(
                        e.associated_funcs.iter().map(
                            |(name, AssociatedFunc { is_pure, params, name_spans, .. })| AssociatedItem {
                                kind: AssociatedItemKind::Func,
                                name: *name,
                                name_span: name_spans[0].clone(),
                                is_pure: Some(*is_pure),
                                params: Some(*params),
                                ..AssociatedItem::default()
                            }
                        )
                    ).chain(
                        e.associated_lets.iter().map(
                            |(name, name_span)| AssociatedItem {
                                kind: AssociatedItemKind::Let,
                                name: *name,
                                name_span: name_span.clone(),
                                ..AssociatedItem::default()
                            }
                        )
                    ).collect(),
                }
            }

            pub fn associated_funcs(&self) -> &HashMap<InternedString, AssociatedFunc> {
                match self {
                    $type_name::Struct(s) => &s.associated_funcs,
                    $type_name::Enum(e) => &e.associated_funcs,
                }
            }

            pub fn associated_lets(&self) -> &HashMap<InternedString, Span> {
                match self {
                    $type_name::Struct(s) => &s.associated_lets,
                    $type_name::Enum(e) => &e.associated_lets,
                }
            }
        }
    };
}

item_shape_impl!(ItemShape);
item_shape_impl!(ItemShapeMut);

impl ItemShapeMut<'_> {
    pub fn associated_funcs_mut(&mut self) -> &mut HashMap<InternedString, AssociatedFunc> {
        match self {
            ItemShapeMut::Struct(s) => &mut s.associated_funcs,
            ItemShapeMut::Enum(e) => &mut e.associated_funcs,
        }
    }

    pub fn associated_lets_mut(&mut self) -> &mut HashMap<InternedString, Span> {
        match self {
            ItemShapeMut::Struct(s) => &mut s.associated_lets,
            ItemShapeMut::Enum(e) => &mut e.associated_lets,
        }
    }
}
