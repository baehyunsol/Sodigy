use crate::{SolvePolyResult, Solver, TypeError};
use sodigy_mir::{Session, Type};
use sodigy_span::Span;
use std::collections::hash_map::{Entry, HashMap};

impl Solver {
    pub fn monomorphize(&mut self, session: &mut Session) -> Result<(), ()> {
        let poly_solver = self.init_poly_solvers(session)?;
        let mut generic_calls: HashMap<Span, GenericCall> = HashMap::new();
        let mut has_error = false;

        for type_var in self.type_vars.keys() {
            match type_var {
                Type::GenericInstance { call, generic } => {
                    let r#type = session.generic_instances.get(&(*call, *generic)).unwrap().clone();

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

        if generic_calls.is_empty() {
            return Ok(());
        }

        // It's key is the call span,
        // and the value is the def_span of the monomorphized function.
        let mut mono_map: HashMap<Span, Span> = HashMap::new();

        for (_, generic_call) in generic_calls.iter() {
            match self.try_solve_poly(&session.polys, &poly_solver, generic_call) {
                SolvePolyResult::NotPoly => {
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
                    mono_map.insert(generic_call.call, p);
                },
                r => panic!("TODO: {r:?}"),
            }
        }

        // TODO: apply mono_map and loop

        if has_error {
            Err(())
        }

        else {
            Ok(())
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
