use super::{Func, LocalValueKey, MaybeInit};
use crate::error::MirError;
use crate::expr::{Expr, ExprKind};
use crate::session::MirSession;
use crate::walker::walker_expr;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use std::collections::HashMap;

// invariants: `HashMap`s do not contain `LocalValueRef::zero`
pub struct LocalValueGraph {
    references: HashMap<LocalValueKey, LocalValueRef>,  // its value referencing other local values
    ref_by: HashMap<LocalValueKey, LocalValueRef>,
    ref_by_ret_val: LocalValueRef,

    // only generics can be referenced by type annotations. otherwise it's an error (dependent types)
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
#[derive(Clone, Copy)]
pub struct LocalValueRef {
    pub must: u32,
    pub cond: u32,
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
    // it draws all the graphs from scratch
    // it's safe to call this function multiple times, but it's not very efficient
    pub fn init_local_value_dependency_graphs(&mut self, session: &mut MirSession) {
        let mut ref_by_table: HashMap<LocalValueKey, HashMap<LocalValueKey, LocalValueRef>> = HashMap::new();

        for local_value in self.local_values.values_mut() {
            if !local_value.is_valid {
                continue;
            }

            let mut references = HashMap::new();

            if let MaybeInit::Init(v) = &local_value.value {
                count_local_values(v, &mut references);
            }

            for (key, ref_) in references.iter() {
                match ref_by_table.get_mut(key) {
                    Some(ref_by) => {
                        assert!(ref_by.insert(local_value.key, *ref_).is_none());
                    },
                    None => {
                        let mut new_ref_by = HashMap::new();
                        new_ref_by.insert(local_value.key, *ref_);

                        ref_by_table.insert(*key, new_ref_by);
                    },
                }
            }

            if local_value.graph.is_none() {
                local_value.graph = Some(LocalValueGraph::new());
            }

            local_value.graph.as_mut().unwrap().references = references;
        }

        for (key, ref_by) in ref_by_table.into_iter() {
            match self.local_values.get_mut(&key) {
                Some(local_value) => {
                    if !local_value.is_valid {
                        continue;
                    }

                    local_value.graph.as_mut().unwrap().ref_by = ref_by;
                },
                None => unreachable!(),
            }
        }

        let mut new_map = HashMap::new();
        count_local_values(&self.return_value, &mut new_map);

        self.local_values_reachable_from_return_value = new_map;

        for (key, ref_by_ret_val) in self.local_values_reachable_from_return_value.iter() {
            match self.local_values.get_mut(&key) {
                Some(local_value) => {
                    if !local_value.is_valid {
                        continue;
                    }

                    local_value.graph.as_mut().unwrap().ref_by_ret_val = *ref_by_ret_val;
                },
                None => unreachable!(),
            }
        }

        // TODO: ref_by_type_annot, ref_type_annot
    }

    pub fn reject_recursive_local_values(&self, session: &mut MirSession) -> Result<(), ()> {
        let mut has_error = false;

        for local_value in self.local_values.values() {
            match &local_value.graph {
                Some(graph) => {
                    if graph.references.contains_key(&local_value.key) {
                        has_error = true;
                        let span = get_span_of_local_value(
                            local_value.value.try_unwrap_init().unwrap(),
                            local_value.key,
                        );
                        session.push_error(MirError::recursive_local_value(local_value, span));
                    }
                },
                _ => {},
            }
        }

        // TODO: find cycles in the graph and raise error if found

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn reject_dependent_types(&self, session: &mut MirSession) -> Result<(), ()> {
        todo!()
    }

    // it traverses the graph from the return value, and finds all the local values that are unreachable
    // it assumes that `reject_recursive_local_values` and `reject_dependent_types` are already run
    pub fn warn_unused_local_values(
        &mut self,
        session: &mut MirSession,
        remove_unused_values: bool,
        silent_warnings: bool,
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

struct GetSpanOfLocalValueContext {
    target: LocalValueKey,
    result: Option<SpanRange>,
}

fn get_span_of_local_value(e: &Expr, key: LocalValueKey) -> SpanRange {
    let mut result = GetSpanOfLocalValueContext {
        target: key,
        result: None,
    };
    walker_expr(e, &mut result, &Box::new(get_span_of_local_value_worker), false);

    result.result.unwrap()
}

fn get_span_of_local_value_worker(
    e: &Expr,
    result: &mut GetSpanOfLocalValueContext,
    _: bool,
) {
    match &e.kind {
        _ if result.result.is_some() => { return; },
        ExprKind::LocalValue { key, .. } if *key == result.target => { result.result = Some(e.span) },
        _ => { return; },
    }
}
