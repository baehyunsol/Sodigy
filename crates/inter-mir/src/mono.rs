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

mod r#enum;
mod expr;
mod func;
mod path;
mod pattern;

pub struct MonomorphizePlan {
    // key: call span
    // value: def_span of the monomorphized function
    pub dispatch_map: HashMap<Span, Span>,
    pub monomorphizations: Vec<Monomorphization>,
}

#[derive(Clone, Debug)]
pub struct Monomorphization {
    pub id: u64,
    pub def_span: Span,

    // This is later used for error messages.
    pub call_span: Span,

    pub generics: HashMap<Span, Type>,
}

impl Session {
    pub fn get_mono_plan(&mut self, poly_solver: &HashMap<Span, PolySolver>, mir_session: &MirSession) -> Result<MonomorphizePlan, ()> {
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
                    if self.solved_generic_args.contains(&(call.clone(), generic.clone())) {
                        continue;
                    }

                    let r#type = match self.generic_args.get(&(call.clone(), generic.clone())) {
                        Some(r#type) => {
                            if r#type.has_unsolved_type() {
                                incomplete_generics.insert(call.clone());
                            }

                            r#type.clone()
                        },
                        None => {
                            incomplete_generics.insert(call.clone());
                            type_var.clone()
                        },
                    };

                    match generic_calls.entry(call.clone()) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().generics.insert(generic.clone(), r#type);
                        },
                        Entry::Vacant(e) => {
                            e.insert(GenericCall {
                                call: call.clone(),
                                def: self.generic_to_def_span.get(generic).unwrap().clone(),
                                variant: self.call_to_variant_span.get(call).cloned(),
                                generics: [(generic.clone(), r#type)].into_iter().collect(),
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
            match self.try_solve_poly(poly_solver, generic_call) {
                // If it's not poly-generic, it's just a normal generic (TODO: better naming), so we monomorphize.
                // If default-impl is chosen, ... it's still a generic function, so we monomorphize.
                SolvePolyResult::NotPoly | SolvePolyResult::DefaultImpl(_) => {
                    // We can do monomorphization only if every generic arguments are known.
                    if incomplete_generics.contains(&generic_call.call) {
                        continue;
                    }

                    for generic in generic_call.generics.keys() {
                        self.solved_generic_args.insert((generic_call.call.clone(), generic.clone()));
                    }

                    // We don't monomorphize built_in functions.
                    if self.built_in_funcs.contains(&generic_call.def) {
                        continue;
                    }

                    let mut sorted_generics: Vec<(&Span, &Type)> = generic_call.generics.iter().collect();
                    sorted_generics.sort_by_key(|&(span, _)| span);
                    let sorted_generics: Vec<&Type> = sorted_generics.into_iter().map(|(_, r#type)| r#type).collect();

                    let monomorphization_id = get_monomorphization_id(&generic_call.def, &sorted_generics);
                    monomorphizations.push(Monomorphization {
                        def_span: generic_call.def.clone(),
                        call_span: generic_call.call.clone(),
                        generics: generic_call.generics.clone(),
                        id: monomorphization_id,
                    });

                    let monomorphized_span = match &generic_call.variant {
                        Some(v) => v.monomorphize(monomorphization_id),
                        None => generic_call.def.monomorphize(monomorphization_id),
                    };
                    dispatch_map.insert(generic_call.call.clone(), monomorphized_span);
                },
                SolvePolyResult::NoCandidates => {
                    has_error = true;
                    self.type_errors.push(TypeError::CannotSpecializePolyGeneric {
                        call: generic_call.call.clone(),
                        poly_def: generic_call.def.clone(),
                        generics: generic_call.generics.clone(),
                        num_candidates: 0,
                    });
                },
                SolvePolyResult::OneCandidate(p) => {
                    dispatch_map.insert(generic_call.call.clone(), p);

                    for generic in generic_call.generics.keys() {
                        self.solved_generic_args.insert((generic_call.call.clone(), generic.clone()));
                    }
                },
                SolvePolyResult::MultiCandidates(ps) => {
                    // TODO
                    //    1. `solve_type` loop runs multiple times.
                    //    2. In the first run, it's likely to reach this error because
                    //       there's not enough hints yet.
                    //    3. So, we want to skip this error for the first few times. But how many times?
                    //    4. ...
                    has_error = true;
                    self.type_errors.push(TypeError::MultiplePolyCandidates {
                        call: generic_call.call.clone(),
                        poly_def: generic_call.def.clone(),
                        candidates: ps.clone(),
                    });
                },
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
            info: format!("{}<{generics}>", self.span_to_string(&mono.def_span).unwrap_or(String::from("????"))),
            span: mono.call_span.clone(),
        }
    }
}

// Let's say there're
// `fn add<T, U, V>(a: T, b: U) -> V;`
// and
// `let x = add(3, 4);`
//
// This would be
// ```
// GenericCall {
//     call: span of add in expr,
//     def: span of add in definition,
//     variant: None,
//     generics: { T: Int, U: Int, V: TypeVar(V) },
// }
// ```
//
// If it's an enum, it has to remember both the enum def span and the variant def span.
// So `Ok(3)` would be
// ```
// GenericCall {
//     call: call span of Ok,
//     def: def span of Result,
//     variant: Some(def span of Ok),
//     generics: { T: Int, E: _ },  // `E` must be known at this point
// }
// ```
#[derive(Clone, Debug)]
pub struct GenericCall {
    pub call: Span,
    pub def: Span,
    pub variant: Option<Span>,
    pub generics: HashMap<Span, Type>,
}

fn get_monomorphization_id(def_span: &Span, generics: &[&Type]) -> u64 {
    let mut bytes = vec![];
    bytes.extend(def_span.hash().to_le_bytes());

    for r#type in generics.iter() {
        bytes.extend(r#type.hash().to_le_bytes());
    }

    (hash(&bytes) & 0xffff_ffff_ffff_ffff) as u64
}
