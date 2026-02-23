use crate::Type;
use sodigy_hir::{FuncShape, Poly, StructShape};
use sodigy_inter_hir as inter_hir;
use sodigy_span::Span;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct GlobalContext<'hir, 'mir> {
    pub func_shapes: Option<&'hir HashMap<Span, FuncShape>>,
    pub struct_shapes: Option<&'hir HashMap<Span, StructShape>>,
    pub polys: Option<&'hir HashMap<Span, Poly>>,

    // generic def span to func def span (or struct def span) map
    pub generic_def_span_rev: Option<&'hir HashMap<Span, Span>>,

    pub lang_items: Option<&'hir HashMap<String, Span>>,

    pub types: Option<&'mir HashMap<Span, Type>>,
    pub generic_args: Option<&'mir HashMap<(Span, Span), Type>>,
}

impl<'hir> GlobalContext<'hir, '_> {
    pub fn new() -> GlobalContext<'static, 'static> {
        GlobalContext {
            func_shapes: None,
            struct_shapes: None,
            polys: None,
            generic_def_span_rev: None,
            lang_items: None,
            types: None,
            generic_args: None,
        }
    }

    pub fn from_inter_hir_session(session: &'hir inter_hir::Session) -> GlobalContext<'hir, 'static> {
        GlobalContext {
            func_shapes: Some(&session.func_shapes),
            struct_shapes: Some(&session.struct_shapes),
            polys: Some(&session.polys),
            generic_def_span_rev: Some(&session.generic_def_span_rev),
            lang_items: Some(&session.lang_items),
            types: None,
            generic_args: None,
        }
    }
}
