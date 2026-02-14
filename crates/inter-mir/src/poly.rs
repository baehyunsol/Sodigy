use crate::{ErrorContext, GenericCall, TypeError, TypeSolver};
use sodigy_error::ParamIndex;
use sodigy_hir::Poly;
use sodigy_mir::{Func, Session, Type};
use sodigy_span::Span;
use std::collections::hash_map::{Entry, HashMap};

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

impl TypeSolver {
    pub fn try_solve_poly(
        &mut self,
        polys: &HashMap<Span, Poly>,
        solvers: &HashMap<Span, PolySolver>,
        generic_call: &GenericCall,
    ) -> SolvePolyResult {
        match polys.get(&generic_call.def) {
            Some(poly) => {
                let solver = solvers.get(&poly.name_span).unwrap();
                let candidates = solver.solve(&generic_call.generics, self.lang_items.clone(), self.intermediate_dir.to_string());

                match candidates.len() {
                    0 => {
                        if poly.has_default_impl {
                            SolvePolyResult::DefaultImpl(poly.name_span)
                        }

                        else {
                            SolvePolyResult::NoCandidates
                        }
                    },
                    1 => SolvePolyResult::OneCandidate(candidates[0]),
                    2.. => SolvePolyResult::MultiCandidates(candidates),
                }
            },
            None => SolvePolyResult::NotPoly,
        }
    }

    pub fn init_poly_solvers(&mut self, session: &Session) -> Result<HashMap<Span, PolySolver>, ()> {
        let mut has_error = false;
        let mut result = HashMap::new();
        let index_by_span = session.funcs.iter().enumerate().map(
            |(i, f)| (f.name_span, i)
        ).collect::<HashMap<Span, usize>>();

        // poly: `#[poly] fn add<T, U>(a: T, b: U) -> Int;`
        // impls:
        //     `#[impl(add)] fn add_int(a: Int, b: Int) -> Int;`
        //     `#[impl(add)] fn add_homo<T>(a: T, b: T) -> Int;`
        for (span, poly) in session.polys.iter() {
            let poly_def = &session.funcs[*index_by_span.get(span).unwrap()];

            // poly_type: `Fn(?T, ?U) -> Int`
            let mut poly_type = get_func_type(poly_def, &session.types);
            let mut solver = PolySolver::new();

            // If we don't know the type of `add` (e.g. there's no type annotation),
            // we can't solve anything!
            if let Some(param_index) = poly_type.find_type_var() {
                has_error = true;
                self.errors.push(TypeError::CannotInferPolyGenericParam {
                    poly_span: *span,
                    param_index,
                });
                continue;
            }

            // poly_type_vars: `?T`, `?U`
            let poly_type_vars = poly_def.generics.iter().map(|generic| generic.name_span).collect::<Vec<_>>();
            poly_type.generics_to_type_vars();

            for r#impl in poly.impls.iter() {
                // impl_type: `Fn(Int, Int) -> Int`
                let mut impl_type = get_func_type(&session.funcs[*index_by_span.get(r#impl).unwrap()], &session.types);

                // If we don't know the type of `add_int` (e.g. there's no type annotation),
                // we can't solve for this impl!
                if let Some(param_index) = impl_type.find_type_var() {
                    has_error = true;
                    self.errors.push(TypeError::CannotInferPolyGenericImpl {
                        poly_span: *span,
                        impl_span: *r#impl,
                        param_index,
                    });
                    continue;
                }

                impl_type.generics_to_type_vars();

                match solve_fn_types(
                    &poly_type,
                    &impl_type,
                    &poly_type_vars,
                    *span,
                    *r#impl,
                    session.lang_items.clone(),
                    session.intermediate_dir.clone(),
                ) {
                    // constraints: `?T = Int`, `?U = Int`
                    Ok(constraints) => {
                        solver.impls.insert(*r#impl, constraints);
                    },
                    Err(e) => {
                        has_error = true;
                        self.errors.extend(e.into_iter().map(|e| TypeError::from(e)));
                        continue;
                    },
                }
            }

            if !has_error {
                solver.build_state_machine();
                result.insert(*span, solver);
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
            let all_impls: Vec<Span> = self.impls.keys().map(|r#impl| *r#impl).collect();
            let mut impls_by_generics: HashMap<Span, HashMap<SimpleType, Vec<Span>>> = HashMap::new();
            //                                 ^^^^                          ^^^^
            //                                 |                             |
            //                                (0)                           (1)
            //
            // (0): def_span of generic parameter of `#[poly]`
            // (1): def_span of `#[impl]`

            for (impl_def_span, types) in self.impls.iter() {
                for (generic_def_span, r#type) in types.iter() {
                    let simple_type = SimpleType::from(r#type);

                    match impls_by_generics.entry(*generic_def_span) {
                        Entry::Occupied(mut e) => match e.get_mut().entry(simple_type) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().push(*impl_def_span);
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![*impl_def_span]);
                            },
                        },
                        Entry::Vacant(e) => {
                            e.insert([(simple_type, vec![*impl_def_span])].into_iter().collect());
                        },
                    }
                }
            }

            impls_by_generics = impls_by_generics.into_iter().filter(
                |(_, impls)| !impls.is_empty()
            ).collect();

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
            //         Tuple { arity: 0 }: [eq_tuple0],
            //         Tuple { arity: 1 }: [eq_tuple1],
            //         Tuple { arity: 2 }: [eq_tuple2],
            //         Tuple { arity: 3 }: [eq_tuple3],
            //         Static { def: Int }: [eq_int],
            //     },
            // }
            // ```

            if !impls_by_generics.is_empty() {
                self.state_machine = Some(StateMachine::build(impls_by_generics, &all_impls));
            }
        }
    }

    pub fn solve(
        &self,
        generics: &HashMap<Span, Type>,

        // for tmp type-solver
        lang_items: HashMap<String, Span>,
        intermediate_dir: String,
    ) -> Vec<Span> {
        let mut matched = vec![];
        let impls = self.impls.keys().map(
            |def_span| *def_span
        ).collect::<Vec<_>>();

        'candidates: for candidate in self.state_machine.as_ref().map(
            |state_machine| state_machine.get_candidates(generics)
        ).unwrap_or(&impls) {
            let candidate_types = self.impls.get(candidate).unwrap();
            let mut type_solver = TypeSolver::tmp(lang_items.clone(), intermediate_dir.to_string());
            let mut types = HashMap::new();

            'generics: for (generic_param, r#type) in generics.iter() {
                let candidate_type = match candidate_types.get(generic_param) {
                    Some(r#type) => r#type,
                    None => {
                        continue 'generics;
                    },
                };

                if let Err(()) = type_solver.solve_supertype(
                    candidate_type,
                    r#type,
                    &mut types,

                    // don't care
                    &mut HashMap::new(),
                    true,
                    None,
                    None,
                    ErrorContext::None,
                    false,
                ) {
                    continue 'candidates;
                }
            }

            matched.push(*candidate);
        }

        matched
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
//     Tuple { arity: 1 } => match U {
//         Tuple { arity: 1 } => [foo2],
//         Tuple { arity: 2 } => [foo3],
//         Var => [foo2, foo3],
//         _ => [],
//     },
//     Static { def: Int } => match U {
//         Static { def: Int } => [foo1],
//         Var => [foo1],
//         _ => [],
//     },
//     Var => match U {
//         Static { def: Int } => [foo1],
//         Tuple { arity: 1 } => [foo2],
//         Tuple { arity: 2 } => [foo3],
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
    Static { def: Span },
    Tuple { arity: usize },
    Param { constructor: Span },
    Func { params: usize },
    Var,
}

impl StateMachine {
    pub fn get_candidates<'a, 'b>(&'a self, generics: &'b HashMap<Span, Type>) -> &'a [Span] {
        // TODO: is it always safe to `unwrap`?
        let r#type = generics.get(&self.generic_param).unwrap();
        let simple_type = SimpleType::from(r#type);

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
    pub fn build(mut impls_by_generics: HashMap<Span, HashMap<SimpleType, Vec<Span>>>, impls: &[Span]) -> StateMachine {
        for old_impls in impls_by_generics.values_mut() {
            let mut no_impls = vec![];

            for (simple_type, r#impl) in old_impls.iter_mut() {
                let simple_type = *simple_type;

                // FIXME: O(n^2)
                *r#impl = r#impl.iter().filter(
                    |impl_span| impls.contains(impl_span)
                ).map(
                    |impl_span| *impl_span
                ).collect();

                if r#impl.is_empty() {
                    no_impls.push(simple_type);
                }
            }

            for no_impl in no_impls.iter() {
                old_impls.remove(no_impl);
            }
        }

        let mut generics_by_types: Vec<(Span, usize)> = impls_by_generics.iter().map(
            |(generic_def_span, impls)| (*generic_def_span, impls.len())
        ).collect();
        generics_by_types.sort_by_key(|(_, types_count)| *types_count);
        let (key, _) = *generics_by_types.last().unwrap();
        let impls_by_type = impls_by_generics.remove(&key).unwrap();
        let default = if let Some(impls) = impls_by_type.get(&SimpleType::Var) {
            impls.to_vec()
        } else {
            vec![]
        };
        let branches = impls_by_type.into_iter().map(
            |(simple_type, mut impls): (SimpleType, Vec<Span>)| {
                impls.extend(&default);

                if impls.len() < 2 || impls_by_generics.is_empty() {
                    (simple_type, StateMachineOrLeaves::Leaves(impls))
                }

                else {
                    (simple_type, StateMachineOrLeaves::StateMachine(StateMachine::build(impls_by_generics.clone(), &impls)))
                }
            }
        ).collect();
        let default = if default.len() < 2 || impls_by_generics.is_empty() {
            Box::new(StateMachineOrLeaves::Leaves(default))
        } else {
            Box::new(StateMachineOrLeaves::StateMachine(StateMachine::build(impls_by_generics.clone(), &default)))
        };

        StateMachine {
            generic_param: key,
            branches,
            default,
        }
    }
}

impl From<&Type> for SimpleType {
    fn from(r#type: &Type) -> SimpleType {
        match r#type {
            Type::Static { def_span, .. } => SimpleType::Static { def: *def_span },
            Type::Tuple { args, .. } => SimpleType::Tuple { arity: args.len() },
            Type::Param { constructor_def_span, .. } => SimpleType::Param { constructor: *constructor_def_span },
            Type::Func { params, .. } => SimpleType::Func { params: params.len() },

            // It's okay to do this because `SimpleType::Var` can match any type.
            // `StateMachine` is just for optimization, so false-positives are okay. False-positives might
            // introduce unnecessary checks, but doesn't hurt the correctness.
            // I want to avoid the complexity of subtyping...
            Type::Never(_) => SimpleType::Var,

            Type::GenericParam { .. } |
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
    pub fn find_type_var(&self) -> Option<ParamIndex> {
        for (i, r#type) in self.params.iter().enumerate() {
            if !r#type.get_type_vars().is_empty() {
                return Some(ParamIndex::Param(i));
            }
        }

        if !self.r#return.get_type_vars().is_empty() {
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

    // for tmp type solver
    lang_items: HashMap<String, Span>,
    intermediate_dir: String,
) -> Result<HashMap<Span, Type>, Vec<TypeError>> {
    if poly.params.len() != r#impl.params.len() {
        return Err(vec![TypeError::PolyImplDifferentNumberOfParams {
            poly_params: poly.params.len(),
            poly_span,
            impl_params: r#impl.params.len(),
            impl_span,
        }]);
    }

    let mut type_solver = TypeSolver::tmp(lang_items, intermediate_dir);
    let mut types = HashMap::new();
    let mut errors = vec![];

    for (i, (poly_type, impl_type)) in (0..poly.params.len()).map(
        |i| (&poly.params[i], &r#impl.params[i])
    ).chain(std::iter::once((&poly.r#return, &r#impl.r#return))).enumerate() {
        // Solves `?T = Int`, `?U = Int` and `Int = Int`.
        if let Err(()) = type_solver.solve_supertype(
            &poly_type,
            &impl_type,
            &mut types,

            // We don't care about `generic_args` because the caller
            // guarantees that there's no generic call.
            &mut HashMap::new(),

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
                poly_span,
                impl_type: impl_type.clone(),
                impl_span,
                param_index: if i < poly.params.len() { ParamIndex::Param(i) } else { ParamIndex::Return },
            });
        }
    }

    if errors.is_empty() {
        Ok(type_vars.iter().filter_map(
            |type_var| match types.get(type_var) {
                Some(r#type) => Some((*type_var, r#type.clone())),

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
