use crate::{PolySolver, SolvePolyResult, TypeError, TypeSolver};
use sodigy_mir::{Session, Type};
use sodigy_span::Span;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

pub struct MonomorphizePlan {
    // key: call span
    // value: def_span of the monomorphized function
    pub dispatch_map: HashMap<Span, Span>,
    pub monomorphizations: Vec<Monomorphization>,
}

#[derive(Clone, Debug)]
pub struct Monomorphization {
    pub def_span: Span,
    pub generics: HashMap<Span, Type>,
    pub id: u128,
}

impl TypeSolver<'_, '_> {
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
        // first parameter is an integer.
        let mut incomplete_generics = HashSet::new();

        for type_var in self.type_vars.keys() {
            match type_var {
                Type::GenericArg { call, generic } => {
                    if already_dispatched.contains(&(*call, *generic)) {
                        continue;
                    }

                    let r#type = match session.generic_args.get(&(*call, *generic)) {
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
                                def: *self.global_context.generic_def_span_rev.unwrap().get(generic).unwrap(),
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
        let mut monomorphizations = vec![];

        for (_, generic_call) in generic_calls.iter() {
            match self.try_solve_poly(self.global_context.polys.unwrap(), &poly_solver, generic_call) {
                SolvePolyResult::NotPoly => {
                    if incomplete_generics.contains(&generic_call.call) {
                        continue;
                    }

                    let monomorphization_id = get_monomorphization_id(generic_call.def, &generic_call.generics);
                    let monomorphized_span = generic_call.def.monomorphize(monomorphization_id);
                    monomorphizations.push(Monomorphization {
                        def_span: generic_call.def,
                        generics: generic_call.generics.clone(),
                        id: monomorphization_id,
                    });
                    dispatch_map.insert(generic_call.call, monomorphized_span);
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

        if has_error {
            Err(())
        }

        else {
            Ok(MonomorphizePlan {
                dispatch_map,
                monomorphizations,
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

fn get_monomorphization_id(def_span: Span, generics: &HashMap<Span, Type>) -> u128 {
    let mut hash = def_span.hash() & 0xffff_ffff_ffff_ffff_ffff_ffff;
    let mut generics: Vec<(Span, &Type)> = generics.iter().map(|(s, t)| (*s, t)).collect();
    generics.sort_by_key(|(s, _)| *s);

    for (_, r#type) in generics.iter() {
        hash += r#type.hash() & 0xffff_ffff_ffff_ffff_ffff_ffff;
        hash &= 0xffff_ffff_ffff_ffff_ffff_ffff;
    }

    hash
}
