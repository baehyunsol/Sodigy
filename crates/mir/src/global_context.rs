use crate::Type;
use sodigy_hir::{EnumShape, FuncShape, ItemShape, Poly, StructShape};
use sodigy_inter_hir as inter_hir;
use sodigy_span::{Span, SpanId};
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct GlobalContext<'hir, 'mir> {
    pub func_shapes: Option<&'hir HashMap<Span, FuncShape>>,
    pub struct_shapes: Option<&'hir HashMap<Span, StructShape>>,
    pub enum_shapes: Option<&'hir HashMap<Span, EnumShape>>,
    pub polys: Option<&'hir HashMap<Span, Poly>>,

    // generic def span to func def span (or struct def span) map
    pub generic_def_span_rev: Option<&'hir HashMap<Span, Span>>,

    pub lang_items: Option<&'hir HashMap<String, Span>>,
    pub built_in_funcs: Option<&'hir HashSet<Span>>,

    pub types: Option<Arc<RwLock<HashMap<Span, Type>>>>,
    pub generic_args: Option<&'mir HashMap<(Span, Span), Type>>,
    pub span_string_map: Option<&'mir HashMap<SpanId, InternedString>>,
}

impl<'hir> GlobalContext<'hir, '_> {
    pub fn new() -> GlobalContext<'static, 'static> {
        GlobalContext {
            func_shapes: None,
            struct_shapes: None,
            enum_shapes: None,
            polys: None,
            generic_def_span_rev: None,
            lang_items: None,
            built_in_funcs: None,
            types: None,
            generic_args: None,
            span_string_map: None,
        }
    }

    pub fn from_inter_hir_session(session: &'hir inter_hir::Session) -> GlobalContext<'hir, 'static> {
        GlobalContext {
            func_shapes: Some(&session.func_shapes),
            struct_shapes: Some(&session.struct_shapes),
            enum_shapes: Some(&session.enum_shapes),
            polys: Some(&session.polys),
            generic_def_span_rev: Some(&session.generic_def_span_rev),
            lang_items: Some(&session.lang_items),
            built_in_funcs: Some(&session.built_in_funcs),
            types: None,
            generic_args: None,
            span_string_map: None,
        }
    }

    pub fn get_item_shape(&self, def_span: &Span) -> Option<ItemShape<'hir>> {
        match self.struct_shapes.map(|ss| ss.get(def_span)) {
            Some(Some(struct_shape)) => Some(ItemShape::Struct(struct_shape)),
            _ => match self.enum_shapes.map(|es| es.get(&def_span)) {
                Some(Some(enum_shape)) => Some(ItemShape::Enum(enum_shape)),
                _ => None,
            },
        }
    }

    pub fn get_type(&self, span: &Span) -> Option<Type> {
        match self.types.as_ref().map(|types| types.read()) {
            Some(Ok(types)) => types.get(span).map(|r#type| r#type.clone()),
            Some(Err(_)) => panic!("global context is poisoned"),
            None => panic!("global context is not initialized"),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items {
            Some(lang_items) => match lang_items.get(lang_item) {
                Some(span) => span.clone(),
                None => panic!("lang_item {lang_item:?} not found"),
            },
            None => panic!("lang_items in global_context not initialized!"),
        }
    }
}
