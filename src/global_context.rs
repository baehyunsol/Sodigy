use sodigy_inter_hir as inter_hir;
use sodigy_inter_mir as inter_mir;
use sodigy_mir::{GlobalContext as MirGlobalContext, Type};
use sodigy_span::Span;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct GlobalContext {
    pub inter_hir_session: Option<inter_hir::Session>,
    pub inter_mir_session: Option<inter_mir::Session>,
    pub types: Option<Arc<RwLock<HashMap<Span, Type>>>>,
}

impl GlobalContext {
    pub fn new() -> GlobalContext {
        GlobalContext {
            inter_hir_session: None,
            inter_mir_session: None,
            types: None,
        }
    }

    pub fn mir_global_context<'s>(&'s self) -> MirGlobalContext<'s, 's> {
        MirGlobalContext {
            func_shapes: Some(&self.inter_mir_session.as_ref().unwrap().func_shapes),
            struct_shapes: Some(&self.inter_mir_session.as_ref().unwrap().struct_shapes),
            enum_shapes: Some(&self.inter_mir_session.as_ref().unwrap().enum_shapes),
            polys: Some(&self.inter_mir_session.as_ref().unwrap().polys),
            generic_def_span_rev: Some(&self.inter_mir_session.as_ref().unwrap().generic_def_span_rev),
            lang_items: Some(&self.inter_mir_session.as_ref().unwrap().lang_items),
            types: self.types.clone(),
            generic_args: Some(&self.inter_mir_session.as_ref().unwrap().generic_args),
            span_string_map: Some(&self.inter_mir_session.as_ref().unwrap().span_string_map),
        }
    }
}
