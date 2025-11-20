use crate::{PolySolver, SolvePolyResult, Solver, TypeError, TypeLog};
use sodigy_mir::{Session, Type};
use sodigy_span::Span;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

pub struct MonomorphizePlan {
    // key: call span
    // value: def_span of the monomorphized function
    pub dispatch_map: HashMap<Span, Span>,
}

impl MonomorphizePlan {
    pub fn is_empty(&self) -> bool {
        self.dispatch_map.is_empty()
    }
}

impl Solver {
    pub fn get_mono_plan(
        &mut self,
        poly_solver: &HashMap<Span, PolySolver>,
        already_dispatched: &mut HashSet<(Span /* call */, Span /* generic */)>,
        session: &Session,
    ) -> Result<MonomorphizePlan, ()> {
        let poly_solver = self.init_poly_solvers(session)?;
        let mut generic_calls: HashMap<Span, GenericCall> = HashMap::new();
        let mut has_error = false;

        // We can infer/monomorphize poly generics even if the type info is incomplete.
        // Let's say there's `3 + a` and we don't know the type of `a`. We can still
        // dispatch the `add` poly because there's only one instance of `add` whose
        // first argument is an integer.
        let mut incomplete_generics = HashSet::new();

        for type_var in self.type_vars.keys() {
            match type_var {
                Type::GenericInstance { call, generic } => {
                    if already_dispatched.contains(&(*call, *generic)) {
                        continue;
                    }

                    let r#type = match session.generic_instances.get(&(*call, *generic)) {
                        Some(r#type) => {
                            if !r#type.get_type_vars().is_empty() {
                                incomplete_generics.insert(*call);
                            }

                            r#type.clone()
                        },
                        None => {
                            incomplete_generics.insert(*call);
                            type_var.clone()
                        },
                    };

                    match generic_calls.entry(*call) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().generics.insert(*generic, r#type);
                        },
                        Entry::Vacant(e) => {
                            e.insert(GenericCall {
                                call: *call,
                                def: *session.generic_def_span_rev.get(generic).unwrap(),
                                generics: [(*generic, r#type)].into_iter().collect(),
                            });
                        },
                    }
                },
                _ => {},
            }
        }

        // Its key is the call span,
        // and the value is the def_span of the monomorphized function.
        let mut dispatch_map: HashMap<Span, Span> = HashMap::new();

        for (_, generic_call) in generic_calls.iter() {
            match self.try_solve_poly(&session.polys, &poly_solver, generic_call) {
                SolvePolyResult::NotPoly => {
                    if incomplete_generics.contains(&generic_call.call) {
                        continue;
                    }

                    // a normal generic function
                    panic!("TODO: {generic_call:?}")
                },
                SolvePolyResult::NoCandidates => {
                    has_error = true;
                    self.errors.push(TypeError::CannotSpecializePolyGeneric {
                        call: generic_call.call,
                        poly_def: generic_call.def,
                        generics: generic_call.generics.clone(),
                        num_candidates: 0,
                    });
                },
                SolvePolyResult::DefaultImpl(p) |
                SolvePolyResult::OneCandidate(p) => {
                    for generic in generic_call.generics.keys() {
                        already_dispatched.insert((generic_call.call, *generic));
                    }

                    dispatch_map.insert(generic_call.call, p);
                },
                r => panic!("TODO: {r:?}"),
            }
        }

        if let Some(log) = &mut self.log {
            for (call, def) in dispatch_map.iter() {
                let mut generic_call = generic_calls.get(call).unwrap().clone();
                let mut generics = generic_call.generics.clone().into_iter().filter(
                    |(span, _)| !already_dispatched.contains(&(generic_call.call, *span))
                ).collect::<Vec<_>>();
                generics.sort_by_key(|(span, _)| *span);
                log.push(TypeLog::Dispatch {
                    call: *call,
                    def: *def,
                    generics,
                });
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(MonomorphizePlan {
                dispatch_map,
            })
        }
    }
}

// Let's say there're
// `fn add<T, U, V>(a: T, b: U) -> V;`
// and
// `let x = add(3, 4);`
//
// This would be
// `GenericCall { call: span_of_add_in_expr, def: span_of_add_in_definition, generics: { T: Int, U: Int, V: TypeVar(V) } }`
#[derive(Clone, Debug)]
pub struct GenericCall {
    pub call: Span,
    pub def: Span,
    pub generics: HashMap<Span, Type>,
}
