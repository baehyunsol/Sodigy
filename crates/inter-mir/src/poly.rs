use crate::{ErrorContext, GenericCall, Session, TypeError, write_log};
use sodigy_error::ParamIndex;
use sodigy_mir::{Func, Session as MirSession, Type};
use sodigy_span::{Span, SpanId};
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

#[cfg(feature = "log")]
use crate::LogEntry;

mod dump;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use dump::RenderStateMachine;

// ```
// #[poly]
// fn add<T, U>(a: T, b: U) -> Int;
//
// #[impl(add)]
// fn add_int(a: Int, b: Int) -> Int;
//
// #[impl(add)]
// fn add_homo<T>(a: T, b: T) -> Int;
//```
//
// The solver does 2 things:
//     1. Are `add_int` and `add_homo` valid implementations for `add`?
//        That said, is `Fn(Int, Int) -> Int` a subtype of `Fn(?T, ?U) -> Int`?
//     2. When given `let x: Int = add("", "");`, should it call `add_int` or `add_homo`?
//        The most naive way is to check `Fn(String, String) -> Int` against every impl
//        of the poly, but we're doing some kinda optimization here.

impl Session {
    pub fn try_solve_poly(
        &mut self,
        solvers: &HashMap<Span, PolySolver>,
        generic_call: &GenericCall,
    ) -> SolvePolyResult {
        let r = match self.polys.get(&generic_call.def) {
            Some(poly) => {
                let solver = solvers.get(&poly.name_span).unwrap();
                let candidates = solver.solve(&generic_call.generics, self);

                match candidates.len() {
                    0 => {
                        if poly.has_default_impl {
                            SolvePolyResult::DefaultImpl(poly.name_span.clone())
                        }

                        else {
                            SolvePolyResult::NoCandidates
                        }
                    },
                    1 => SolvePolyResult::OneCandidate(candidates[0].clone()),
                    2.. => SolvePolyResult::MultiCandidates(candidates),
                }
            },
            None => SolvePolyResult::NotPoly,
        };

        write_log!(self, LogEntry::TrySolvePoly {
            generic_call: generic_call.clone(),
            poly_def: self.polys.get(&generic_call.def).cloned(),
            result: r.clone(),
        });
        r
    }

    pub fn init_poly_solvers(&mut self, mir_session: &MirSession) -> Result<HashMap<Span, PolySolver>, ()> {
        let mut has_error = false;
        let mut result = HashMap::new();
        let index_by_span = mir_session.funcs.iter().enumerate().map(
            |(i, f)| (f.name_span.clone(), i)
        ).collect::<HashMap<Span, usize>>();

        // poly: `#[poly] fn add<T, U>(a: T, b: U) -> Int;`
        // impls:
        //     `#[impl(add)] fn add_int(a: Int, b: Int) -> Int;`
        //     `#[impl(add)] fn add_homo<T>(a: T, b: T) -> Int;`
        for (span, poly) in self.polys.iter() {
            let poly_def = &mir_session.funcs[*index_by_span.get(span).unwrap()];

            // poly_type: `Fn(?T, ?U) -> Int`
            let mut poly_type = get_func_type(poly_def, &self.types);
            let mut solver = PolySolver::new();

            // If we don't know the type of `add` (e.g. there's no type annotation),
            // we can't solve anything!
            if let Some(param_index) = poly_type.find_unsolved_type() {
                has_error = true;
                self.type_errors.push(TypeError::CannotInferPolyGenericParam {
                    poly_span: span.clone(),
                    param_index,
                });
                continue;
            }

            // poly_type_vars: `?T`, `?U`
            let poly_type_vars = poly_def.generics.iter().map(|generic| generic.name_span.clone()).collect::<Vec<_>>();
            poly_type.generics_to_type_vars();

            for r#impl in poly.impls.iter() {
                // impl_type: `Fn(Int, Int) -> Int`
                let mut impl_type = get_func_type(&mir_session.funcs[*index_by_span.get(r#impl).unwrap()], &self.types);

                // If we don't know the type of `add_int` (e.g. there's no type annotation),
                // we can't solve for this impl!
                if let Some(param_index) = impl_type.find_unsolved_type() {
                    has_error = true;
                    self.type_errors.push(TypeError::CannotInferPolyGenericImpl {
                        poly_span: span.clone(),
                        impl_span: r#impl.clone(),
                        param_index,
                    });
                    continue;
                }

                impl_type.generics_to_type_vars();

                match solve_fn_types(
                    &poly_type,
                    &impl_type,
                    &poly_type_vars,
                    span.clone(),
                    r#impl.clone(),
                    self,
                ) {
                    // constraints: `?T = Int`, `?U = Int`
                    Ok(mut constraints) => {
                        constraints = constraints.into_iter().map(
                            |(span, mut r#type)| {
                                r#type.type_var_to_generic_param();
                                (span, r#type)
                            }
                        ).collect();
                        solver.impls.insert(r#impl.clone(), constraints);
                    },
                    Err(mut e) => {
                        has_error = true;
                        self.type_errors.extend(e.drain(..));
                        continue;
                    },
                }
            }

            if !has_error {
                solver.build_state_machine();
                result.insert(span.clone(), solver);
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(result)
        }
    }
}

#[derive(Clone, Debug)]
pub struct PolySolver {
    pub impls: HashMap<Span, HashMap<Span, Type>>,
    //                 ^^^^          ^^^^  ^^^^
    //                 |             |     |
    //                (0)           (1)   (2)
    //
    // (0): def_span of `#[impl]`
    // (1): def_span of generic parameter of `#[poly]`
    // (2): this generic parameter has to have this `Type`, in order to match this `#[impl]`

    // This is just for an optimization. It reduces the number of candidates to check.
    // So, it can return false-positives, but not false-negatives.
    pub state_machine: Option<StateMachine>,
}

impl PolySolver {
    pub fn new() -> PolySolver {
        PolySolver {
            impls: HashMap::new(),
            state_machine: None,
        }
    }

    pub fn build_state_machine(&mut self) {
        // There's no need for an optimization!
        if self.impls.len() < 2 {
            self.state_machine = None;
        }

        else {
            let all_impls: Vec<Span> = self.impls.keys().cloned().collect();
            let mut generic_params = vec![];
            let mut impls_by_generics: HashMap<Span, HashMap<SimpleType, Vec<Span>>> = HashMap::new();
            //                                 ^^^^                          ^^^^
            //                                 |                             |
            //                                (0)                           (1)
            //
            // (0): def_span of generic parameter of `#[poly]`
            // (1): def_span of `#[impl]`

            for (impl_def_span, types) in self.impls.iter() {
                for (generic_def_span, r#type) in types.iter() {
                    let simple_type = SimpleType::from_type_param(r#type);

                    if simple_type == SimpleType::GenericParam {
                        generic_params.push((impl_def_span.clone(), generic_def_span.clone(), r#type.get_generic_param_def_span()));
                        continue;
                    }

                    match impls_by_generics.entry(generic_def_span.clone()) {
                        Entry::Occupied(mut e) => match e.get_mut().entry(simple_type) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().push(impl_def_span.clone());
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![impl_def_span.clone()]);
                            },
                        },
                        Entry::Vacant(e) => {
                            e.insert([(simple_type, vec![impl_def_span.clone()])].into_iter().collect());
                        },
                    }
                }
            }

            // ```
            // #[poly] fn foo<T1, T2>(x: T1, y: T2);
            // #[impl(foo)] fn foo1<T3>(x: T3, y: T3);
            // ```
            // In this case, we can do a further optimization. We can use the fact that
            // `x` and `y` in `foo1` have the same type.
            let mut same_generic_params_: HashMap<Span, Vec<(Span, Span)>> = HashMap::new();
            //                                    ^^^^       ^^^^  ^^^^
            //                                    |          |     |
            //                                   (0)        (1)   (2)
            //
            // (0): T3
            // (1): foo1
            // (2): T1 and T2
            //
            // So it would look like `{ T3: [(foo1, T1), (foo1, T2)] }`.
            // It means "T1 and T2 of foo1 must be the same".

            for (impl_def_span, generic_def_span, generic_param) in generic_params.iter() {
                if let Some(generic_param) = generic_param {
                    match same_generic_params_.entry(generic_param.clone()) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().push((impl_def_span.clone(), generic_def_span.clone()));
                        },
                        Entry::Vacant(e) => {
                            e.insert(vec![(impl_def_span.clone(), generic_def_span.clone())]);
                        },
                    }
                }
            }

            // We reduce `same_generic_params_`.
            let mut same_generic_params: HashMap<(Span, Span), Vec<Span>> = HashMap::new();
            //                                    ^^^^  ^^^^       ^^^^
            //                                    |     |          |
            //                                   (0)   (1)        (2)
            //
            // (0): foo1
            // (1): T3
            // (2): T1 and T2
            //
            // So it would look like `{ (foo1, T3): [T1, T2] }`.
            // It means "T1 and T2 of foo1 must be the same".

            for (impl_generic_param_def_span, params) in same_generic_params_.into_iter() {
                for (impl_def_span, generic_def_span) in params.into_iter() {
                    match same_generic_params.entry((impl_def_span.clone(), impl_generic_param_def_span.clone())) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().push(generic_def_span);
                        },
                        Entry::Vacant(e) => {
                            e.insert(vec![generic_def_span]);
                        },
                    }
                }
            }

            // ```
            // #[poly] fn foo<T1, T2>(x: T1, y: T2) -> Int;
            //
            // #[impl(foo)] fn foo1<T3>(x: T3, y: Char) -> Int;
            // #[impl(foo)] fn foo2(x: Int, y: Int) -> Int;
            // ```
            //
            // In the above example, `generic_params` would be `[(span of foo1, span of T1)]`.
            // That means "regardless of type of T1, foo1 always matches".
            for (impl_def_span, generic_def_span, _) in generic_params.into_iter() {
                match impls_by_generics.entry(generic_def_span) {
                    Entry::Occupied(mut e) => {
                        for impls in e.get_mut().values_mut() {
                            impls.push(impl_def_span.clone());
                        }

                        match e.get_mut().entry(SimpleType::GenericParam) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().push(impl_def_span);
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![impl_def_span]);
                            },
                        }
                    },
                    Entry::Vacant(e) => {
                        e.insert([(SimpleType::GenericParam, vec![impl_def_span])].into_iter().collect());
                    },
                }
            }

            for impls in impls_by_generics.values_mut() {
                impls.insert(SimpleType::Var, all_impls.clone());

                if let Some(impls) = impls.get_mut(&SimpleType::GenericParam) {
                    impls.sort();
                    impls.dedup();
                }
            }

            // ```sodigy
            // #[poly] fn eq<T>(_: T, _: T) -> Bool;
            // #[impl(eq)] fn eq_tuple0(_: (), _: ()) -> Bool;
            // #[impl(eq)] fn eq_tuple1<T0>(_: (T0,), _: (T0,)) -> Bool;
            // #[impl(eq)] fn eq_tuple2<T0, T1>(_: (T0, T1), _: (T0, T1)) -> Bool;
            // #[impl(eq)] fn eq_tuple3<T0, T1, T2>(_: (T0, T1, T2), _: (T0, T1, T2)) -> Bool;
            // #[impl(eq)] fn eq_int(_: Int, _: Int) -> Bool;
            // // ... a lot more impls
            // ```
            //
            // in the above code, `impls_by_generics` looks like below:
            //
            // ```rust
            // {
            //     T: {
            //         Data { constructor: Tuple, arity: 0 }: [eq_tuple0],
            //         Data { constructor: Tuple, arity: 1 }: [eq_tuple1],
            //         Data { constructor: Tuple, arity: 2 }: [eq_tuple2],
            //         Data { constructor: Tuple, arity: 3 }: [eq_tuple3],
            //         Data { constructor: Int, arity: 0 }: [eq_int],
            //     },
            // }
            // ```

            let mut state_machine = StateMachine::build(impls_by_generics, all_impls.into_iter().collect());

            for ((impl_def_span, _), generic_def_spans) in same_generic_params.into_iter() {
                if generic_def_spans.len() < 2 {
                    continue;
                }

                apply_same_generic_params(
                    &mut state_machine,
                    &impl_def_span,
                    &generic_def_spans,
                    None,
                    false,
                );
            }

            let mut state_machine = StateMachineOrLeaves::StateMachine(state_machine);
            state_machine.optimize();

            if let StateMachineOrLeaves::StateMachine(state_machine) = state_machine {
                self.state_machine = Some(state_machine);
            }
        }
    }

    pub fn solve(
        &self,
        generics: &HashMap<Span, Type>,

        // It's for a tmp-session.
        session: &Session,
    ) -> Vec<Span> {
        let mut matched = vec![];
        let impls = self.impls.keys().cloned().collect::<Vec<_>>();

        'candidates: for candidate in self.state_machine.as_ref().map(
            |state_machine| state_machine.get_candidates(generics)
        ).unwrap_or(&impls) {
            let candidate_types = self.impls.get(candidate).unwrap();
            let mut tmp_session = Session::tmp(session);

            'generics: for (generic_param, r#type) in generics.iter() {
                let mut candidate_type = match candidate_types.get(generic_param) {
                    Some(r#type) => r#type.clone(),
                    None => {
                        continue 'generics;
                    },
                };
                candidate_type.generic_to_type_var();

                if let Err(()) = tmp_session.solve_supertype(
                    &candidate_type,
                    r#type,
                    true,
                    None,
                    None,
                    ErrorContext::None,
                    false,
                ) {
                    continue 'candidates;
                }
            }

            matched.push(candidate.clone());
        }

        matched
    }
}

fn apply_same_generic_params(
    state_machine: &mut StateMachine,
    impl_def_span: &Span,
    generic_def_spans: &[Span],

    // `generic_def_span` must have this type.
    target_type: Option<SimpleType>,

    // if it's set, we'll remove `impl_def_span` when we reach the leaves
    mut unreachable: bool,
) {
    match (generic_def_spans.contains(&state_machine.generic_param), target_type) {
        (true, None) => {
            for (r#type, state_machine) in state_machine.branches.iter_mut() {
                match (*r#type, state_machine) {
                    (_, StateMachineOrLeaves::Leaves(_)) => {
                        // no problem
                    },
                    (t @ SimpleType::Data { .. }, StateMachineOrLeaves::StateMachine(s)) => {
                        apply_same_generic_params(s, impl_def_span, generic_def_spans, Some(t), false);
                    },
                    (_, StateMachineOrLeaves::StateMachine(s)) => {
                        apply_same_generic_params(s, impl_def_span, generic_def_spans, None, false);
                    },
                }
            }
        },
        (true, Some(target_type)) => {
            for (r#type, state_machine) in state_machine.branches.iter_mut() {
                match (*r#type, state_machine, unreachable) {
                    (_, StateMachineOrLeaves::Leaves(leaves), true) => {
                        *leaves = leaves.iter().filter(
                            |span| *span != impl_def_span
                        ).cloned().collect();
                    },
                    (t @ SimpleType::Data { .. }, StateMachineOrLeaves::Leaves(leaves), _) => {
                        if t != target_type {
                            *leaves = leaves.iter().filter(
                                |span| *span != impl_def_span
                            ).cloned().collect();
                        }
                    },
                    (_, StateMachineOrLeaves::Leaves(_), false) => {},
                    (t, StateMachineOrLeaves::StateMachine(s), _) => {
                        if let SimpleType::Data { .. } = t && t != target_type {
                            unreachable = true;
                        }

                        apply_same_generic_params(s, impl_def_span, generic_def_spans, Some(target_type), unreachable);
                    },
                }
            }
        },
        (false, _) => {
            for state_machine in state_machine.branches.values_mut() {
                match state_machine {
                    StateMachineOrLeaves::Leaves(leaves) if unreachable => {
                        *leaves = leaves.iter().filter(
                            |span| *span != impl_def_span
                        ).cloned().collect();
                    },
                    StateMachineOrLeaves::Leaves(_) => {},
                    StateMachineOrLeaves::StateMachine(s) => {
                        apply_same_generic_params(s, impl_def_span, generic_def_spans, target_type, unreachable);
                    },
                }
            }
        },
    }
}

// ```sodigy
// #[poly] fn foo<T, U>(_: T, _: U);
// #[impl(foo)] fn foo1(_: Int, _: Int);
// #[impl(foo)] fn foo2<T0>(_: (T0,), _: (T0,));
// #[impl(foo)] fn foo3<T1, T2>(_: (T1,), _: (T1, T2));
// ```
//
// From the above example, we'll create a state machine that looks like below.
//
// ```
// // 1. We don't care about default `#[impl]` here. We'll check the default impl when nothing matches.
// // 2. This isn't the actual matching. This is just to reduce the number of candidates.
// //    There's an actual poly-solver that runs on the candidates. The poly-solver instantiates a
// //    tmp type-solver and compares the types.
// match T {
//     Data { constructor: Tuple, arity: 1 } => match U {
//         Data { constructor: Tuple, arity: 1 } => [foo2],
//         Data { constructor: Tuple, arity: 2 } => [foo3],
//         Var => [foo2, foo3],
//         _ => [],
//     },
//     Data { constructor: Int, arity: 0 } => match U {
//         Data { constructor: Int, arity: 0 } => [foo1],
//         Var => [foo1],
//         _ => [],
//     },
//     Var => match U {
//         Data { constructor: Int, arity: 0 } => [foo1],
//         Data { constructor: Tuple, arity: 1 } => [foo2],
//         Data { constructor: Tuple, arity: 2 } => [foo3],
//         Var => [foo1, foo2, foo3],
//         _ => [],
//     },
//     _ => [],
// }
// ```
#[derive(Clone, Debug)]
pub struct StateMachine {
    pub generic_param: Span,
    pub branches: HashMap<SimpleType, StateMachineOrLeaves>,

    // If nothing in `branches` matches, this default branch is matched.
    pub default: Box<StateMachineOrLeaves>,
}

#[derive(Clone, Debug)]
pub enum StateMachineOrLeaves {
    StateMachine(StateMachine),
    Leaves(Vec<Span>),  // def_span of `#[impl]`s
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SimpleType {
    Data {
        // base of def_span
        constructor: SpanId,
        arity: usize,
    },
    Func { params: usize },
    GenericParam,
    Var,
}

impl StateMachine {
    pub fn get_candidates<'a, 'b>(&'a self, generics: &'b HashMap<Span, Type>) -> &'a [Span] {
        // TODO: is it always safe to `unwrap`?
        let r#type = generics.get(&self.generic_param).unwrap();
        let simple_type = SimpleType::from_type_arg(r#type);

        match self.branches.get(&simple_type) {
            Some(StateMachineOrLeaves::StateMachine(s)) => s.get_candidates(generics),
            Some(StateMachineOrLeaves::Leaves(impls)) => impls,
            None => match self.default.as_ref() {
                StateMachineOrLeaves::StateMachine(s) => s.get_candidates(generics),
                StateMachineOrLeaves::Leaves(impls) => impls,
            },
        }
    }

    // read the comments in `PolySolver::build_state_machine()`
    pub fn build(
        mut impls_by_generics: HashMap<Span, HashMap<SimpleType, Vec<Span>>>,
        all_impls: HashSet<Span>,
    ) -> StateMachine {
        // ```
        // #[poly] fn foo<T1, T2>(x: T1, y: T2) -> Int;
        //
        // #[impl(foo)] fn foo1<T3>(x: T3, y: Char) -> Int;
        // #[impl(foo)] fn foo2(x: Int, y: Int) -> Int;
        // #[impl(foo)] fn foo3(x: Char, y: Char) -> Int;
        // ```
        //
        // `impls_by_generics` looks like below:
        //
        // ```
        // {
        //     "T1": {
        //         GenericParam: [foo1],
        //         Int: [foo1, foo2],
        //         Char: [foo1, foo3],
        //         TypeVar: [foo1, foo2, foo3],
        //     },
        //     "T2": {
        //         Char: [foo1, foo3],
        //         Int: [foo2],
        //         TypeVar: [foo1, foo2, foo3],
        //     },
        // }
        // ```

        match impls_by_generics.len() {
            0 => unreachable!(),
            1 => {
                let (generic_param, branches) = impls_by_generics.iter().next().unwrap();
                let default_branch = match branches.get(&SimpleType::GenericParam) {
                    Some(impls) => StateMachineOrLeaves::Leaves(intersect_and_sort(impls, &all_impls)),
                    None => StateMachineOrLeaves::Leaves(vec![]),
                };

                StateMachine {
                    generic_param: generic_param.clone(),
                    branches: branches.iter().filter(
                        |(r#type, _)| **r#type != SimpleType::GenericParam
                    ).map(
                        |(r#type, impls)| (
                            *r#type,
                            StateMachineOrLeaves::Leaves(intersect_and_sort(impls, &all_impls)),
                        )
                    ).collect(),
                    default: Box::new(default_branch),
                }
            },
            _ => {
                let generic_param = {
                    // 1. It prefers a state machine without `SimpleType::GenericParam`.
                    // 2. It prefers a state machine with more number of branches.
                    let mut classify = vec![];

                    for (generic_param, branches) in impls_by_generics.iter() {
                        classify.push((
                            generic_param,
                            branches.contains_key(&SimpleType::GenericParam),
                            branches.len(),
                        ));
                    }

                    classify.sort_by_key(|(_, _, n)| usize::MAX - *n);
                    classify.sort_by_key(|(_, g, _)| *g as usize);
                    classify[0].0.clone()
                };

                let branches = impls_by_generics.remove(&generic_param).unwrap();
                let default_branch = match branches.get(&SimpleType::GenericParam) {
                    Some(impls) => StateMachineOrLeaves::StateMachine(StateMachine::build(impls_by_generics.clone(), intersect_and_sort(impls, &all_impls).into_iter().collect())),
                    None => StateMachineOrLeaves::Leaves(vec![]),
                };

                StateMachine {
                    generic_param,
                    branches: branches.iter().filter(
                        |(r#type, _)| *r#type != &SimpleType::GenericParam
                    ).map(
                        |(r#type, impls)| {
                            let impls_by_generics = impls_by_generics.clone();
                            let impls = intersect_and_sort(impls, &all_impls);

                            (
                                *r#type,
                                StateMachineOrLeaves::StateMachine(StateMachine::build(impls_by_generics, impls.into_iter().collect())),
                            )
                        }
                    ).collect(),
                    default: Box::new(default_branch),
                }
            },
        }
    }
}

impl StateMachineOrLeaves {
    pub fn optimize(&mut self) {
        // ```
        // Data(byte, 0) => match GenericParam(0) {
        //     Var => [poly-impl-1],
        //     Data(byte, 0) => [poly-impl-1],
        //     Data(int, 0) => [poly-impl-1],
        //     _ => [poly-impl-1],
        // },
        // ```
        // when every node has the same leaves
        // ->
        // ```
        // Data(byte, 0) => [poly-impl-1],
        // ```
        //
        // ```
        // Data(int, 0) => match GenericParam(0) {
        //     Var => [poly-impl-2],
        //     Data(int, 0) => [poly-impl-2],
        //     Data(byte, 0) => [],
        //     _ => [],
        // },
        // ```
        // when a branch and a default branch have the same leaves (even if the branch is `SimpleType::Var`)
        // ->
        // ```
        // Data(int, 0) => match GenericParam(0) {
        //     Var => [poly-impl-2],
        //     Data(int, 0) => [poly-impl-2],
        //     _ => [],
        // },
        // ```
        if let StateMachineOrLeaves::StateMachine(s) = self {
            for branch in s.branches.values_mut() {
                branch.optimize();
            }

            s.default.optimize();

            if let StateMachineOrLeaves::Leaves(leaves) = s.default.as_ref() {
                let leaves = leaves.to_vec();
                let mut unnecessary_branches = vec![];

                for (r#type, branch) in s.branches.iter() {
                    // We can compare the leaves because it's sorted!
                    if let StateMachineOrLeaves::Leaves(b_leaves) = branch && b_leaves == &leaves {
                        unnecessary_branches.push(*r#type);
                    }
                }

                for r#type in unnecessary_branches.iter() {
                    s.branches.remove(r#type);
                }

                if s.branches.is_empty() {
                    *self = StateMachineOrLeaves::Leaves(leaves);
                }
            }
        }
    }
}

// We sort the result in order for easy comparison.
fn intersect_and_sort(spans: &[Span], superset: &HashSet<Span>) -> Vec<Span> {
    let mut s = spans.iter().filter(
        |span| superset.contains(span)
    ).cloned().collect::<Vec<_>>();
    s.sort();
    s
}

impl SimpleType {
    fn from_type_param(r#type: &Type) -> SimpleType {
        match r#type {
            Type::Data { constructor_def_span, args, .. } => SimpleType::Data {
                constructor: constructor_def_span.id().unwrap(),
                arity: args.as_ref().map(|args| args.len()).unwrap_or(0),
            },
            Type::Func { params, .. } => SimpleType::Func { params: params.len() },
            Type::GenericParam { .. } => SimpleType::GenericParam,

            // We can treat these like generic params, in order to
            // make the implementation of the state machines simpler.
            // It'd introduce some false-positives, but that's fine.
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => SimpleType::GenericParam,
        }
    }

    fn from_type_arg(r#type: &Type) -> SimpleType {
        match r#type {
            Type::Data { constructor_def_span, args, .. } => SimpleType::Data {
                constructor: constructor_def_span.id().unwrap(),
                arity: args.as_ref().map(|args| args.len()).unwrap_or(0),
            },
            Type::Func { params, .. } => SimpleType::Func { params: params.len() },
            Type::GenericParam { .. } => unreachable!(),

            // It's okay to do this because `SimpleType::Var` can match any type.
            // `StateMachine` is just for optimization, so false-positives are okay. False-positives might
            // introduce unnecessary checks, but doesn't hurt the correctness.
            // I want to avoid the complexity of subtyping...
            Type::Never(_) => SimpleType::Var,

            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => SimpleType::Var,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SolvePolyResult {
    NotPoly,
    DefaultImpl(Span),
    NoCandidates,
    OneCandidate(Span),
    MultiCandidates(Vec<Span>),
}

#[derive(Clone, Debug)]
pub struct FuncType {
    pub params: Vec<Type>,
    pub r#return: Type,
}

impl FuncType {
    // If it finds a type var, it returns the index of the param that has the type var.
    pub fn find_unsolved_type(&self) -> Option<ParamIndex> {
        for (i, r#type) in self.params.iter().enumerate() {
            if r#type.has_unsolved_type() {
                return Some(ParamIndex::Param(i));
            }
        }

        if self.r#return.has_unsolved_type() {
            return Some(ParamIndex::Return);
        }

        None
    }

    pub fn generics_to_type_vars(&mut self) {
        for r#type in self.params.iter_mut().chain(std::iter::once(&mut self.r#return)) {
            r#type.generic_to_type_var();
        }
    }
}

fn get_func_type(f: &Func, types: &HashMap<Span, Type>) -> FuncType {
    let (r#return, mut params) = match types.get(&f.name_span) {
        Some(Type::Func { r#return, params, .. }) => (*r#return.clone(), params.clone()),
        _ => unreachable!(),
    };

    for param in params.iter_mut() {
        match param {
            Type::Var { def_span, is_return: false } => match types.get(def_span) {
                Some(Type::Var { .. }) | None => {},
                Some(v) => {
                    *param = v.clone();
                },
            },
            _ => {},
        }
    }

    FuncType {
        params,
        r#return,
    }
}

// poly: `#[poly] fn add<T, U>(a: T, b: U) -> Int;`
// impl: `#[impl(add)] fn add_int(a: Int, b: Int) -> Int;`
fn solve_fn_types(
    // `Fn(?T, ?U) -> Int`
    poly: &FuncType,

    // `Fn(Int, Int) -> Int`
    r#impl: &FuncType,

    // `T`, `U`
    type_vars: &[Span],

    poly_span: Span,
    impl_span: Span,

    // It's used to create a tmp-session.
    session: &Session,
) -> Result<HashMap<Span, Type>, Vec<TypeError>> {
    if poly.params.len() != r#impl.params.len() {
        return Err(vec![TypeError::PolyImplDifferentNumberOfParams {
            poly_params: poly.params.len(),
            poly_span,
            impl_params: r#impl.params.len(),
            impl_span,
        }]);
    }

    let mut tmp_session = Session::tmp(session);
    let mut errors = vec![];

    for (i, (poly_type, impl_type)) in (0..poly.params.len()).map(
        |i| (&poly.params[i], &r#impl.params[i])
    ).chain(std::iter::once((&poly.r#return, &r#impl.r#return))).enumerate() {
        // Solves `?T = Int`, `?U = Int` and `Int = Int`.
        if let Err(()) = tmp_session.solve_supertype(
            poly_type,
            impl_type,

            // Below 5 arguments are for error messages.
            // Since we're creating new error messages, we don't care about these.
            true,
            None,
            None,
            ErrorContext::None,
            false,
        ) {
            errors.push(TypeError::CannotImplPoly {
                poly_type: poly_type.clone(),
                poly_span: poly_span.clone(),
                impl_type: impl_type.clone(),
                impl_span: impl_span.clone(),
                param_index: if i < poly.params.len() { ParamIndex::Param(i) } else { ParamIndex::Return },
            });
        }
    }

    if errors.is_empty() {
        Ok(type_vars.iter().filter_map(
            |type_var| match tmp_session.types.get(type_var) {
                Some(r#type) => Some((type_var.clone(), r#type.clone())),

                // Let's say `poly` has 2 generic parameters `T` and `U` but
                // `U` is not used anywhere. Then this parameter won't be in `types`
                // and has nothing to do with solving this poly.
                None => None,
            }
        ).collect())
    }

    else {
        Err(errors)
    }
}
