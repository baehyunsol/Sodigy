use crate::{ErrorContext, GenericCall, Solver};
use sodigy_hir::Poly;
use sodigy_mir::{Func, Session, Type};
use sodigy_span::Span;
use std::collections::HashMap;

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

impl Solver {
    pub fn try_solve_poly(
        &mut self,
        polys: &HashMap<Span, Poly>,
        solvers: &HashMap<Span, PolySolver>,
        generic_call: &GenericCall,
    ) -> SolvePolyResult {
        match polys.get(&generic_call.def) {
            Some(poly) => {
                let solver = solvers.get(&poly.name_span).unwrap();
                let candidates = solver.solve(&generic_call.generics);

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
            let def_func = &session.funcs[*index_by_span.get(span).unwrap()];

            // def_type: `Fn(?T, ?U) -> Int`
            let mut def_type = get_func_type(def_func, &session.types);
            let mut solver = PolySolver::new();

            // If we don't know the type of `add` (e.g. there's no type annotation),
            // we can't solve anything!
            if def_type.has_type_var() {
                has_error = true;
                self.errors.push(Error {});
                continue;
            }

            // def_type_vars: `?T`, `?U`
            let def_type_vars = def_func.generics.iter().map(|generic| generic.name_span).collect::<Vec<_>>();
            def_type.generics_to_type_vars();

            for r#impl in poly.impls.iter() {
                // impl_type: `Fn(Int, Int) -> Int`
                let mut impl_type = get_func_type(&session.funcs[*index_by_span.get(r#impl).unwrap()], &session.types);

                // If we don't know the type of `add_int` (e.g. there's no type annotation),
                // we can't solve for this impl!
                if impl_type.has_type_var() {
                    has_error = true;
                    self.errors.push(Error {});
                    continue;
                }

                impl_type.generics_to_type_vars();

                match solve_fn_types(&def_type, &impl_type, &def_type_vars, session.lang_items.clone()) {
                    // constraints: `?T = Int`, `?U = Int`
                    Ok(constraints) => {
                        solver.impls.insert(*r#impl, constraints);
                    },
                    Err(e) => {
                        has_error = true;
                        self.errors.push(Error {});  // TODO: TypeError::from(e)
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
pub enum Constraint {
    Eq {
        type_var: Span,

        // It must be a concrete type: no variable at all!
        r#type: Type,
    },
}

pub enum CheckResult {
    Pass,
    Maybe,
    Fail,
}

impl Constraint {
    pub fn check(&self, generics: &HashMap<Span, Type>) -> CheckResult {
        match self {
            Constraint::Eq { type_var, r#type: constraint } => match generics.get(type_var) {
                Some(call) => match (call, constraint) {
                    (Type::Static { def_span: c1, .. }, Type::Static { def_span: c2, .. }) => {
                        if c1 == c2 {
                            CheckResult::Pass
                        }

                        else {
                            CheckResult::Fail
                        }
                    },
                    (Type::Unit(_), Type::Unit(_)) => CheckResult::Pass,
                    (Type::Never(_), Type::Never(_)) => CheckResult::Pass,
                    (Type::Param { .. }, Type::Param { .. }) => todo!(),
                    (Type::Func { .. }, Type::Func { .. }) => todo!(),
                    (Type::Var { .. } | Type::GenericInstance { .. }, _) => CheckResult::Maybe,
                    (
                        Type::Static { .. } | Type::Unit(_) | Type::Never(_) | Type::Param { .. } | Type::Func { .. },
                        Type::Static { .. } | Type::Unit(_) | Type::Never(_) | Type::Param { .. } | Type::Func { .. },
                    ) => CheckResult::Fail,
                    _ => panic!("TODO: {:?}", (call, constraint)),
                },
                None => CheckResult::Maybe,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct PolySolver {
    pub impls: HashMap<Span, Vec<Constraint>>,
}

impl PolySolver {
    pub fn new() -> PolySolver {
        PolySolver {
            impls: HashMap::new(),
        }
    }

    pub fn build_state_machine(&mut self) {
        // TODO: I want to build a state machine to optimize the poly-solving.
        //       But it runs without the state machine anyway, so I'll come back here later.
    }

    pub fn solve(&self, generics: &HashMap<Span, Type>) -> Vec<Span> {
        let mut candidates = vec![];

        // TODO: I want to optimize it with a state machine...
        //       For now, we're just naively checking everything!
        for (r#impl, constraints) in self.impls.iter() {
            let mut failed = false;

            for constraint in constraints.iter() {
                if let CheckResult::Fail = constraint.check(generics) {
                    failed = true;
                    break;
                }
            }

            if !failed {
                candidates.push(*r#impl);
            }
        }

        candidates
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
pub enum SolvePolyError {
    DifferentNumberOfArgs,
    CannotImplPoly {
        def_type: Type,
        impl_type: Type,

        // If it's None, it's the return value.
        param_index: Option<usize>,
    },
}

pub struct FuncType {
    pub params: Vec<Type>,
    pub r#return: Type,
}

impl FuncType {
    pub fn has_type_var(&self) -> bool {
        for r#type in self.params.iter().chain(std::iter::once(&self.r#return)) {
            if !r#type.get_type_vars().is_empty() {
                return true;
            }
        }

        false
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

// def: `#[poly] fn add<T, U>(a: T, b: U) -> Int;`
// impl: `#[impl(add)] fn add_int(a: Int, b: Int) -> Int;`
fn solve_fn_types(
    // `Fn(?T, ?U) -> Int`
    def: &FuncType,

    // `Fn(Int, Int) -> Int`
    r#impl: &FuncType,

    // `T`, `U`
    type_vars: &[Span],

    // for the type solver
    lang_items: HashMap<String, Span>,
) -> Result<Vec<Constraint>, Vec<SolvePolyError>> {
    if def.params.len() != r#impl.params.len() {
        return Err(vec![SolvePolyError::DifferentNumberOfArgs]);
    }

    let mut solver = Solver::new(lang_items, false);
    let mut types = HashMap::new();
    let mut constraints = vec![];
    let mut errors = vec![];

    for (i, (def_type, impl_type)) in (0..def.params.len()).map(
        |i| (&def.params[i], &r#impl.params[i])
    ).chain(std::iter::once((&def.r#return, &r#impl.r#return))).enumerate() {
        // Solves `?T = Int`, `?U = Int` and `Int = Int`.
        if let Err(()) = solver.solve_subtype(
            &def_type,
            &impl_type,
            &mut types,

            // We don't care about `generic_instances` because the caller
            // guarantees that there's no generic instance.
            &mut HashMap::new(),

            // Below 4 arguments are for error messages.
            // Since we're creating new error messages, we don't care about these.
            true,
            None,
            None,
            ErrorContext::None,
        ) {
            errors.push(SolvePolyError::CannotImplPoly {
                def_type: def_type.clone(),
                impl_type: impl_type.clone(),
                param_index: if i < def.params.len() { Some(i) } else { None },
            });
        }
    }

    let mut maybe_free_vars = vec![];

    for type_var in type_vars.iter() {
        match types.get(type_var) {
            Some(r#type) => match r#type {
                // Assume `def` has generics `T` and `U` and `impl` has a generic `V`.
                // If `?T = Int` and `?U = ?V`, `?U` is free.
                // If `?T = ?V` and `?U = ?V`, we have another constraint: `?T = ?U`.
                Type::Var { .. } => {
                    maybe_free_vars.push(*type_var);
                },
                _ => {
                    if !r#type.get_type_vars().is_empty() {
                        todo!()  // what should I do here?
                    }

                    else {
                        constraints.push(Constraint::Eq { type_var: *type_var, r#type: r#type.clone() });
                    }
                },
            },
            // `def` has generics `T` and `U`, but `U` is not used anywhere,
            // then `types.get(?U)` would return `None`.
            // That's programmer's mistake, but not the business of the PolySolver.
            None => {
                // no constraints
            },
        }
    }

    // TODO: check free variables

    if errors.is_empty() {
        Ok(constraints)
    }

    else {
        Err(errors)
    }
}
