use crate::{PolySolver, Session, SolvePolyResult, TypeError};
use sodigy_endec::Endec;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir_all,
    exists,
    join4,
    parent,
    write_bytes,
};
use sodigy_mir::{Session as MirSession, Type};
use sodigy_span::{MonomorphizationInfo, Span};
use sodigy_string::hash;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

mod expr;
mod func;
mod pattern;

pub struct MonomorphizePlan {
    // key: call span
    // value: def_span of the monomorphized function
    pub dispatch_map: HashMap<Span, Span>,
    pub monomorphizations: Vec<Monomorphization>,
}

#[derive(Clone, Debug)]
pub struct Monomorphization {
    pub id: u128,
    pub def_span: Span,

    // This is later used for error messages.
    pub call_span: Span,

    pub generics: HashMap<Span, Type>,
}

impl Session {
    pub fn get_mono_plan(&mut self, poly_solver: &HashMap<Span, PolySolver>, mir_session: &MirSession) -> Result<MonomorphizePlan, ()> {
        // TODO: `get_mono_plan` is called multiple times, but I don't think we have to call `init_poly_solvers` multiple times
        let poly_solver = self.init_poly_solvers(mir_session)?;

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
                    if self.solved_generic_args.contains(&(*call, *generic)) {
                        continue;
                    }

                    let r#type = match self.generic_args.get(&(*call, *generic)) {
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
                                def: *self.generic_def_span_rev.get(generic).unwrap(),
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
            match self.try_solve_poly(&poly_solver, generic_call) {
                SolvePolyResult::NotPoly => {
                    // We can do monomorphization only if every generic arguments are known.
                    if incomplete_generics.contains(&generic_call.call) {
                        continue;
                    }

                    for generic in generic_call.generics.keys() {
                        self.solved_generic_args.insert((generic_call.call, *generic));
                    }

                    // We don't monomorphize built_in functions.
                    if self.built_in_funcs.contains(&generic_call.def) {
                        continue;
                    }

                    let monomorphization_id = get_monomorphization_id(generic_call.def, &generic_call.generics);
                    let monomorphized_span = generic_call.def.monomorphize(monomorphization_id);
                    monomorphizations.push(Monomorphization {
                        def_span: generic_call.def,
                        call_span: generic_call.call,
                        generics: generic_call.generics.clone(),
                        id: monomorphization_id,
                    });
                    dispatch_map.insert(generic_call.call, monomorphized_span);
                },
                SolvePolyResult::NoCandidates => {
                    has_error = true;
                    self.type_errors.push(TypeError::CannotSpecializePolyGeneric {
                        call: generic_call.call,
                        poly_def: generic_call.def,
                        generics: generic_call.generics.clone(),
                        num_candidates: 0,
                    });
                },
                // It's still a generic function, so we have to monomorphize this.
                SolvePolyResult::DefaultImpl(_) => {
                    for generic in generic_call.generics.keys() {
                        self.solved_generic_args.insert((generic_call.call, *generic));
                    }

                    // We don't monomorphize built_in functions.
                    if self.built_in_funcs.contains(&generic_call.def) {
                        continue;
                    }

                    let monomorphization_id = get_monomorphization_id(generic_call.def, &generic_call.generics);
                    let monomorphized_span = generic_call.def.monomorphize(monomorphization_id);
                    monomorphizations.push(Monomorphization {
                        def_span: generic_call.def,
                        call_span: generic_call.call,
                        generics: generic_call.generics.clone(),
                        id: monomorphization_id,
                    });
                    dispatch_map.insert(generic_call.call, monomorphized_span);
                },
                SolvePolyResult::OneCandidate(p) => {
                    dispatch_map.insert(generic_call.call, p);

                    for generic in generic_call.generics.keys() {
                        self.solved_generic_args.insert((generic_call.call, *generic));
                    }
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

    pub fn store_monomorphization_info(&self) -> Result<(), FileError> {
        for mono in self.monomorphizations.values() {
            let mono_info = self.render_monomorphization_info(mono);
            let id_str = format!("{:x}", mono_info.id);
            let mono_info_at = join4(
                &self.intermediate_dir,
                "mono",
                id_str.get(0..2).unwrap(),
                id_str.get(2..).unwrap(),
            )?;

            if !exists(&parent(&mono_info_at)?) {
                create_dir_all(&parent(&mono_info_at)?)?;
            }

            write_bytes(
                &mono_info_at,
                &mono_info.encode(),
                WriteMode::CreateOrTruncate,
            )?;
        }

        Ok(())
    }

    pub fn render_monomorphization_info(&self, mono: &Monomorphization) -> MonomorphizationInfo {
        let mut generics = mono.generics.iter().collect::<Vec<_>>();
        generics.sort_by_key(|(span, _)| *span);
        let generics = generics.iter().map(
            |(_, r#type)| self.render_type(r#type)
        ).collect::<Vec<_>>().join(", ");

        MonomorphizationInfo {
            id: mono.id,
            parent: None,  // TODO: track parents
            info: format!("{}<{generics}>", self.span_to_string(mono.def_span).unwrap_or(String::from("????"))),
            span: mono.call_span,
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
    let mut bytes = vec![];
    bytes.extend(def_span.hash().to_le_bytes());

    let mut generics: Vec<(Span, &Type)> = generics.iter().map(|(s, t)| (*s, t)).collect();
    generics.sort_by_key(|(s, _)| *s);

    for (_, r#type) in generics.iter() {
        bytes.extend(r#type.hash().to_le_bytes());
    }

    hash(&bytes) & 0xffff_ffff_ffff_ffff_ffff_ffff
}
