use sodigy_inter_hir as inter_hir;
use sodigy_mir::{GlobalContext as MirGlobalContext, Type};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct GlobalContext {
    pub inter_hir_session: Option<inter_hir::Session>,
    pub types: Option<HashMap<Span, Type>>,
    pub generic_args: Option<HashMap<(Span, Span), Type>>,
}

impl GlobalContext {
    pub fn new() -> GlobalContext {
        GlobalContext {
            inter_hir_session: None,
            types: None,
            generic_args: None,
        }
    }

    pub fn mir_global_context<'s>(&'s self) -> MirGlobalContext<'s, 's> {
        MirGlobalContext {
            func_shapes: Some(&self.inter_hir_session.as_ref().unwrap().func_shapes),
            struct_shapes: Some(&self.inter_hir_session.as_ref().unwrap().struct_shapes),
            enum_shapes: Some(&self.inter_hir_session.as_ref().unwrap().enum_shapes),
            polys: Some(&self.inter_hir_session.as_ref().unwrap().polys),
            generic_def_span_rev: Some(&self.inter_hir_session.as_ref().unwrap().generic_def_span_rev),
            lang_items: Some(&self.inter_hir_session.as_ref().unwrap().lang_items),
            types: Some(self.types.as_ref().unwrap()),
            generic_args: Some(self.generic_args.as_ref().unwrap()),
        }
    }
}
