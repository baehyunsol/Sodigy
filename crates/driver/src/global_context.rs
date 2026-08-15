use sodigy_inter_hir as inter_hir;
use sodigy_inter_mir as inter_mir;
use sodigy_mir::{GlobalContext as MirGlobalContext, Type};
use sodigy_span::Span;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Let's say there are N workers.
// 1. One of the workers will initialize inter_hir_session and store it to disk.
// 2. The other workers will load the inter_hir_session from the disk.
// 3. Every worker is in the mir stage. They need the inter_hir_session, and
//    each has a copy of the inter_hir_session. They can even modify their copy
//    of the inter_hir_session, but the modifications are not shared between
//    workers.
// 4. Same for inter_mir_session.
// 5. When loading the inter_mir_session, the workers also load the `types`.
// 6. The `types` are shared. Any worker can update the `types` and the updates
//    are propagated immediately.
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
            generic_to_def_span: Some(&self.inter_mir_session.as_ref().unwrap().generic_to_def_span),
            variant_to_enum_span: Some(&self.inter_mir_session.as_ref().unwrap().variant_to_enum_span),
            lang_items: Some(&self.inter_mir_session.as_ref().unwrap().lang_items),
            built_in_funcs: Some(&self.inter_mir_session.as_ref().unwrap().built_in_funcs),
            types: self.types.clone(),
            generic_args: Some(&self.inter_mir_session.as_ref().unwrap().generic_args),
            span_string_map: Some(&self.inter_mir_session.as_ref().unwrap().span_string_map),
        }
    }
}
