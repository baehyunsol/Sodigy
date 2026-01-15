pub(crate) use sodigy_mir::{Expr, Type};
use sodigy_mir::Session as MirSession;
use std::collections::{HashMap, HashSet};

mod endec;
mod error;
mod mono;
mod poly;
mod session;
mod type_solver;

pub use error::{ErrorContext, ExprContext, TypeError, type_error_to_general_error};
pub(crate) use mono::GenericCall;
pub(crate) use poly::{PolySolver, SolvePolyResult};
pub use session::Session;
use type_solver::TypeSolver;

pub fn solve_type(mut mir_session: MirSession) -> (Session, MirSession) {
    let mut has_error = false;
    let mut type_solver = TypeSolver::new(mir_session.lang_items.clone(), mir_session.intermediate_dir.clone());
    let mut poly_solver = HashMap::new();

    // It does 2 things.
    // 1. It prevents the compiler from dispatching the same call (with the same dispatch) multiple times.
    // 2. If a call is dispatched, we shouldn't throw `CannotInferGeneric` error for the call.
    //    -> this happens for poly generics. You can dispatch a poly generic with partially infered types!
    let mut dispatched_calls = HashSet::new();

    loop {
        for func in mir_session.funcs.iter() {
            // We'll check generic functions after monomorphization.
            if func.generics.is_empty() && !func.built_in {
                if let (_, true) = type_solver.solve_func(func, &mut mir_session.types, &mut mir_session.generic_instances) {
                    has_error = true;
                }
            }
        }

        for r#let in mir_session.lets.iter() {
            let mut impure_calls = vec![];

            if let (_, true) = type_solver.solve_let(
                r#let,
                &mut impure_calls,
                &mut mir_session.types,
                &mut mir_session.generic_instances,
            ) {
                has_error = true;
            }

            if !impure_calls.is_empty() {
                type_solver.errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: r#let.keyword_span,
                    context: r#let.origin.into(),
                });
            }
        }

        for assert in mir_session.asserts.iter() {
            let mut impure_calls = vec![];

            if let Err(()) = type_solver.solve_assert(
                assert,
                &mut impure_calls,
                &mut mir_session.types,
                &mut mir_session.generic_instances,
            ) {
                has_error = true;
            }

            if !impure_calls.is_empty() {
                type_solver.errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: assert.keyword_span,
                    context: ExprContext::TopLevelAssert,
                });
            }
        }

        // TODO: structs and enums

        // We don't want to do monomorphization if there's a type error
        // -> an erroneous monomorphization might generate very unreadable error messages
        if has_error {
            break;
        }

        // If we initialize it at every iteration, that'd be too expensive.
        // If we initialize it before the first iteration, we have too small type information to use.
        if poly_solver.is_empty() {
            poly_solver = match type_solver.init_poly_solvers(&mir_session) {
                Ok(s) => s,
                Err(()) => {
                    has_error = true;
                    break;
                },
            };
        }

        match type_solver.get_mono_plan(&poly_solver, &mut dispatched_calls, &mir_session) {
            Ok(mono) => {
                if mono.is_empty() {
                    break;
                }

                else {
                    mir_session.dispatch(&mono.dispatch_map);
                    // TODO: do we have to invalidate previous `generic_instances` after dispatching?
                }
            },
            Err(()) => {
                has_error = true;
                break;
            },
        }
    }

    // If we already have an error, it's likely that type-inference is not complete,
    // and there's no point to check whether the type-inference is complete.
    if !has_error {
        type_solver.apply_never_types(
            &mut mir_session.types,
            &mut mir_session.generic_instances,
        );

        if let Err(()) = type_solver.check_all_types_infered(
            &mir_session.types,
            &mir_session.generic_instances,
            &mir_session.generic_def_span_rev,
            &dispatched_calls,
        ) {
            has_error = true;
        }

        // If the solver has failed to infer some types, it's dangerous to check type assertions.
        // Checking type assertions may solve type variables, which may introduce false-positives.
        else if let Err(()) = type_solver.check_type_assertions(
            &mir_session.type_assertions,
            &mut mir_session.types,
            &mut mir_session.generic_instances,
        ) {
            has_error = true;
        }
    }

    for warning in type_solver.warnings.iter() {
        mir_session.warnings.push(type_error_to_general_error(warning, &mir_session));
    }

    if has_error {
        // In order to create error messages, we have to convert spans to strings.
        // But that's very expensive operation, so we initialize this map only when there's an error.
        mir_session.init_span_string_map();

        for error in type_solver.errors.iter() {
            mir_session.errors.push(type_error_to_general_error(error, &mir_session));
        }
    }

    // It's relatively cheap. It'll be stored in cache-dir.
    let inter_mir_session = Session {
        types: mir_session.types.clone(),
        generic_instances: mir_session.generic_instances.clone(),
        errors: mir_session.errors.clone(),
        warnings: mir_session.warnings.clone(),
    };

    (inter_mir_session, mir_session)
}
