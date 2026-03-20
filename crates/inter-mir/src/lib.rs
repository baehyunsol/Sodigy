// It's defined here because it defines a macro.
mod log;

use crate::log::write_log;
use crate::mono::GenericCall;
use sodigy_error::Error;
use sodigy_mir::{Expr, Session as MirSession, Type};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

mod endec;
mod error;
mod mono;
mod poly;
mod session;
mod span_string_map;
mod type_solver;

pub use error::{ErrorContext, ExprContext, TypeError};
pub use log::LogEntry;
pub use mono::Monomorphization;
pub(crate) use poly::{PolySolver, SolvePolyResult};
pub use session::Session;

#[cfg(test)]
pub(crate) use poly::RenderStateMachine;

// There are 2 sessions and it's a mess.
// 1. The function reads/updates `.funcs`, `.lets` and `.asserts` of `mir_session`.
// 2. The function reads `.type_assertions` of `mir_session`.
// 3. The function doesn't read/update any other field of `mir_session`.
pub fn solve_type(mir_session: &mut MirSession<'_, '_>) -> Session {
    let mut has_error = false;
    let mut session = Session::from_mir_session(mir_session);
    let mut poly_solver = HashMap::new();
    let mut prev_blocked_type_var_count = usize::MAX;

    // There's nothing to solve for structs and enums.
    // Their type information is collected by `Struct::from_hir` and `Enum::from_hir`.

    for i in 0..32 {
        if i == 31 {
            has_error = true;

            // It means there's an infinite loop in inter-mir...
            // I don't think we should set recursion-limit for this.
            // If there's an infinite loop in the user code (not in the sodigy compiler),
            // that must be caught eariler.
            session.errors.push(Error::ice(132301, Span::None));
            break;
        }

        session.blocked_type_vars = HashSet::new();

        for func in mir_session.funcs.iter() {
            // We'll check generic functions after monomorphization.
            if func.generics.is_empty() && !func.built_in {
                if let (_, true) = session.solve_func(func) {
                    has_error = true;
                }
            }
        }

        for r#let in mir_session.lets.iter() {
            let mut impure_calls = vec![];

            if let (_, true) = session.solve_let(r#let, &mut impure_calls) {
                has_error = true;
            }

            if !impure_calls.is_empty() {
                session.type_errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: r#let.keyword_span,
                    context: r#let.origin.into(),
                });
            }
        }

        for assert in mir_session.asserts.iter() {
            let mut impure_calls = vec![];

            if let Err(()) = session.solve_assert(assert, &mut impure_calls) {
                has_error = true;
            }

            if !impure_calls.is_empty() {
                session.type_errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: assert.keyword_span,
                    context: ExprContext::TopLevelAssert,
                });
            }
        }

        // We don't want to do monomorphization if there's a type error
        // -> an erroneous monomorphization might generate very unreadable error messages
        if has_error {
            break;
        }

        // If we initialize it at every iteration, that'd be too expensive.
        // If we initialize it before the first iteration, we have too small type information to use.
        if poly_solver.is_empty() {
            poly_solver = match session.init_poly_solvers(&mir_session) {
                Ok(s) => s,
                Err(()) => {
                    has_error = true;
                    break;
                },
            };

            for (span, solver) in poly_solver.iter() {
                write_log!(session, LogEntry::InitPolySolver {
                    poly_def_span: *span,
                    solver: solver.clone(),
                });
            }
        }

        match session.get_mono_plan(&poly_solver, mir_session) {
            Ok(mut plan) => {
                for monomorphization in plan.monomorphizations.drain(..) {
                    if session.monomorphizations.contains_key(&monomorphization.id) {
                        continue;
                    }

                    write_log!(session, LogEntry::Monomorphization(monomorphization.clone()));

                    if let Some(index) = session.funcs_rev.get(&monomorphization.def_span) {
                        let func = &mir_session.funcs[*index];
                        let new_func = session.monomorphize_func(func, &monomorphization);
                        session.monomorphizations.insert(monomorphization.id, monomorphization);
                        session.func_shapes.insert(new_func.name_span, new_func.shape());
                        session.funcs_rev.insert(new_func.name_span, mir_session.funcs.len());
                        mir_session.funcs.push(new_func);
                    }

                    else {
                        // maybe a struct or an enum?
                        todo!()
                    }
                }

                if !plan.dispatch_map.is_empty() {
                    let mut generic_args = HashMap::new();
                    mir_session.dispatch(
                        &plan.dispatch_map,
                        &session.associated_funcs.iter().map(
                            |AssociatedFuncInstance { def_span, call_span, .. }| (
                                *call_span,
                                *def_span,
                            )
                        ).collect(),
                        &session.func_shapes,
                        &mut generic_args,
                    );
                    session.associated_funcs.clear();

                    for ((call, generic), r#type) in generic_args.into_iter() {
                        session.add_type_var(r#type.clone(), None);
                        session.generic_args.insert((call, generic), r#type);
                    }

                    continue;
                }
            },
            Err(()) => {
                has_error = true;
                break;
            },
        }

        // Oops, we have a blocked type var, so we cannot finish the pass.
        // A blocked type var is a type var that "is too difficult to solve now, but maybe
        // able to solve when we have more information".
        if session.blocked_type_vars.len() > 0 {
            // we're making a progress! let's continue
            if session.blocked_type_vars.len() < prev_blocked_type_var_count {
                prev_blocked_type_var_count = session.blocked_type_vars.len();
                continue;
            }

            // we can't solve the types even with more information. let's just give up and ask the programmer
            // to give more type annotations
            else {
                for def_span in session.blocked_type_vars.iter() {
                    session.type_errors.push(TypeError::CannotInferType {
                        id: None,
                        span: *def_span,
                        is_return: false,
                    });
                }

                has_error = true;
            }
        }

        break;
    }

    // If we already have an error, it's likely that type-inference is not complete,
    // and there's no point to check whether the type-inference is complete.
    if !has_error {
        session.apply_never_types();

        if let Err(()) = session.check_all_types_infered() {
            // has_error = true;
        }

        // If the solver has failed to infer some types, it's dangerous to check type assertions.
        // Checking type assertions may solve type variables, which may introduce false-positives.
        else if let Err(()) = session.check_type_assertions(&mir_session.type_assertions) {
            // has_error = true;
        }
    }

    // FIXME: It's too expensive...
    session.init_span_string_map(
        &mir_session.lets,
        &mir_session.funcs,
        &mir_session.structs,
        &mir_session.enums,
        &mir_session.asserts,
        &mir_session.aliases,
    );

    for warning in session.type_warnings.iter() {
        session.warnings.push(session.type_error_to_general_error(warning));
    }

    for error in session.type_errors.iter() {
        session.errors.push(session.type_error_to_general_error(error));
    }

    session
}

#[derive(Clone, Debug)]
pub struct AssociatedFuncInstance {
    field_name: InternedString,

    // def_span of `associated_func::unwrap::3`,
    // which looks like `Span::Poly { name: intern("associated_func::unwrap::3"), kind: PolySpanKind::Name }`
    def_span: Span,

    // span of `unwrap` in `x.y.z.unwrap()`
    call_span: Span,
}
