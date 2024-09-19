use super::{Func, LocalValueKey, MaybeInit};
use crate::expr::{Expr, ExprKind};
use crate::session::MirSession;
use crate::walker::walker_expr;
use std::collections::HashMap;

// invariants: `HashMap`s do not contain `LocalValueRef::zero`
pub struct LocalValueGraph {
    references: HashMap<LocalValueKey, LocalValueRef>,  // its value referencing other local values
    ref_by: HashMap<LocalValueKey, LocalValueRef>,
    ref_by_ret_val: LocalValueRef,

    // type annotations can reference local values (syntactically), but that's an error (semantically)
    // hir pass is supposed to catch all those errors, but I count it again here because
    // 1. a safe guard
    // 2. I might allow dependent types someday
    ref_by_type_annot: LocalValueRef,
    ref_type_annot: HashMap<LocalValueKey, LocalValueRef>,  // its type annotation referencing other local values
}

impl LocalValueGraph {
    pub fn new() -> Self {
        LocalValueGraph {
            references: HashMap::new(),
            ref_by: HashMap::new(),
            ref_by_ret_val: LocalValueRef::zero(),
            ref_by_type_annot: LocalValueRef::zero(),
            ref_type_annot: HashMap::new(),
        }
    }
}

// this local value is un-conditionally referenced at least `must` times
// and conditionally referenced at least `cond` times.
// if both are 0, it's guaranteed that this value is not referenced
#[derive(Clone)]
pub struct LocalValueRef {
    must: u32,
    cond: u32,
}

impl LocalValueRef {
    pub fn zero() -> Self {
        LocalValueRef {
            must: 0,
            cond: 0,
        }
    }
}

impl Func {
    pub fn init_local_value_dependency_graphs(&mut self, session: &mut MirSession) {
        for local_value in self.local_values.values_mut() {
            let mut references = HashMap::new();

            if let MaybeInit::Init(v) = &local_value.value {
                count_local_values(v, &mut references);
            }

            if local_value.graph.is_none() {
                local_value.graph = Some(LocalValueGraph::new());
            }

            local_value.graph.as_mut().unwrap().references = references;
        }

        // TODO: `LocalValueGraph` has 5 fields, but it only initialized 1 of them
        //       there are 4 more to go
    }

    pub fn reject_recursive_local_values(&self, session: &mut MirSession) -> Result<(), ()> {
        todo!()
    }

    pub fn reject_dependent_types(&self, session: &mut MirSession) -> Result<(), ()> {
        todo!()
    }

    pub fn warn_unused_local_values(
        &mut self,
        session: &mut MirSession,
        remove_unused_values: bool,
    ) {
        todo!()
    }
}

fn count_local_values(e: &Expr, result: &mut HashMap<LocalValueKey, LocalValueRef>) {
    walker_expr(e, result, &Box::new(count_local_values_worker), false);
}

fn count_local_values_worker(
    e: &Expr,
    result: &mut HashMap<LocalValueKey, LocalValueRef>,
    is_conditional: bool,
) {
    if let ExprKind::LocalValue { key, .. } = &e.kind {
        let key = *key;
        let mut new_ref = result.get(&key).map(|r| r.clone()).unwrap_or(LocalValueRef::zero());

        if is_conditional {
            new_ref.cond += 1;
        }

        else {
            new_ref.must += 1;
        }

        result.insert(key, new_ref);
    }
}
