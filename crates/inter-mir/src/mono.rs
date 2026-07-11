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
use sodigy_mir::{Session as MirSession, Type, TypeUnit};
use sodigy_span::{MonomorphizationInfo, Span, SpanId};
use sodigy_string::hash;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

mod r#enum;
mod expr;
mod func;
mod path;
mod pattern;
mod r#struct;
mod r#type;

pub struct MonomorphizePlan {
    // key: call span
    // value: def_span of the monomorphized function
    pub dispatch_map: HashMap<Span, Span>,
    pub monomorphizations: Vec<Monomorphization>,

    // extra types to monomorphize
    pub intermediate_types: Vec<(Type, Span /* call_span, for error messages */)>,
}

#[derive(Clone, Debug)]
pub struct Monomorphization {
    pub id: u64,
    pub def_span: Span,

    // This is later used for error messages.
    pub call_span: Span,

    // This is also used for error messages.
    // When you encounter `eq.<Foo<Int>>()`, there are 2 things to monomorphize.
    // 1. `fn eq<T>` with `<T=Foo<Int>>`.
    // 2. `struct Foo<T>` with `T=Int`.
    // The second case is *intermediate*. I'm not sure whether it's a correct term to use...
    pub is_intermediate: bool,

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
        let mut intermediate_types = vec![];

        for generic_call in generic_calls.values() {
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

                    // When we monomorphize `eq<Foo<Int>>(..)`, we also have to monomorphize `Foo<Int>`.
                    //
                    // TODO: I guess it's adding A LOT of duplicate types to `intermediate_types`.
                    //       We have to deduplicate it. Maybe using `.flatten()`?
                    for r#type in sorted_generics.iter() {
                        if r#type.has_to_be_monomorphized() {
                            intermediate_types.push(((*r#type).clone(), generic_call.call.clone()));
                        }
                    }

                    let monomorphization_id = get_monomorphization_id(generic_call.def.id().unwrap(), &sorted_generics).unwrap();
                    monomorphizations.push(Monomorphization {
                        def_span: generic_call.def.clone(),
                        call_span: generic_call.call.clone(),
                        is_intermediate: false,
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
                // If it has multiple candidates, that means either
                // 1. The user wrote wrong Sodigy code and we can't choose one poly-generic impl.
                // 2. There's no problem with the user's code, but we don't have enough information to solve this.
                SolvePolyResult::MultiCandidates(ps) => {
                    self.maybe_type_errors.push(TypeError::MultiplePolyCandidates {
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
                intermediate_types,
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

// Let's say there's `enum Foo<T> = { Message(T), ... };`. The initial name_span of the enum
// would look like `Span::Range(1234)`.
// Let's say it's monomorphized twice: `Foo<Int>` and `Foo<String>`.
// That would create 2 more enum instances (mir::Enum, hir::EnumShape, ...), and
// those 2 would have different name_spans: `Span::Monomorphize { id: 5678, span: Span::Range(1234) }`
// and `Span::Monomorphize { id: 9012, span: Span::Range(1234) }`.
// Sometimes we have to treat `Foo<Int>` and `Foo<String>` differently, and sometimes not.
//
// For example, if we want to know the type of the payload of the first variant, we have to
// look for an `hir::EnumShape`, which is identified by their name_span. We have to use their
// monomorphization_id to treat them differently.
//
// The type-checker has to treat the 2 `Foo`s equally. When it tries to unify `Foo<Int>` and
// `Foo<String>`, it has to know that the two types have the same constructor and different
// type arguments. So, `mir::Type::Data`'s `constructor_def_span` field has type `SpanId`,
// not `Span`.
pub fn get_def_span_from_id(span_id: SpanId, args: &Option<Vec<Type>>) -> Span {
    match args {
        Some(args) => match get_monomorphization_id_owned(span_id, args) {
            Some(mono_id) => Span::Range(span_id).monomorphize(mono_id),
            None => Span::Range(span_id),
        },
        None => Span::Range(span_id),
    }
}

// If there're un-solved type_args, it returns None.
pub fn get_monomorphization_id(def_span: SpanId, type_args: &[&Type]) -> Option<u64> {
    let mut units = vec![TypeUnit::DefSpan(def_span)];

    for type_arg in type_args.iter() {
        units.extend(type_arg.flatten()?);
    }

    Some((hash(&units.encode()) & 0xffff_ffff_ffff_ffff) as u64)
}

// If there're un-solved type_args, it returns None.
pub fn get_monomorphization_id_owned(def_span: SpanId, type_args: &[Type]) -> Option<u64> {
    let mut units = vec![TypeUnit::DefSpan(def_span)];

    for type_arg in type_args.iter() {
        units.extend(type_arg.flatten()?);
    }

    Some((hash(&units.encode()) & 0xffff_ffff_ffff_ffff) as u64)
}
