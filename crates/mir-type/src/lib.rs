pub(crate) use sodigy_mir::{Expr, Type};
use sodigy_mir::Session;
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};
use sodigy_string::unintern_string;
use std::collections::{HashMap, HashSet};

mod error;
mod mono;
mod poly;
mod solver;

pub use error::{ErrorContext, RenderTypeError, TypeError};
pub(crate) use mono::GenericCall;
pub(crate) use poly::{PolySolver, SolvePolyResult};
use solver::Solver;

pub fn solve(mut session: Session) -> (Session, Solver) {
    let mut has_error = false;
    let mut type_solver = Solver::new(session.lang_items.clone());
    let mut poly_solver = HashMap::new();

    // It does 2 things.
    // 1. It prevents the compiler from dispatching the same call (with the same dispatch) multiple times.
    // 2. If a call is dispatched, we shouldn't throw `CannotInferGeneric` error for the call.
    //    -> this happens for poly generics. You can dispatch a poly generic with partially infered types!
    let mut dispatched_calls = HashSet::new();

    loop {
        for func in session.funcs.iter() {
            // We'll check generic functions after monomorphization.
            if func.generics.is_empty() && !func.built_in {
                if let (_, true) = type_solver.solve_func(func, &mut session.types, &mut session.generic_instances) {
                    has_error = true;
                }
            }
        }

        for r#let in session.lets.iter() {
            if let (_, true) = type_solver.solve_let(r#let, &mut session.types, &mut session.generic_instances) {
                has_error = true;
            }
        }

        for assert in session.asserts.iter() {
            if let Err(()) = type_solver.solve_assert(assert, &mut session.types, &mut session.generic_instances) {
                has_error = true;
            }
        }

        // TODO: structs and enums

        // If we initialize it at every iteration, that'd be too expensive.
        // If we initialize it before the first iteration, we have too small type information to use.
        if poly_solver.is_empty() {
            poly_solver = match type_solver.init_poly_solvers(&session) {
                Ok(s) => s,
                Err(()) => {
                    has_error = true;
                    break;
                },
            };
        }

        match type_solver.get_mono_plan(&poly_solver, &mut dispatched_calls, &session) {
            Ok(mono) => {
                if mono.is_empty() {
                    break;
                }

                else {
                    session.dispatch(&mono.dispatch_map);
                    // TODO: do we have to invalidate previous `generic_instances` after dispatching?
                }
            },
            Err(()) => {
                has_error = true;
                break;
            },
        }
    }

    type_solver.apply_never_types(
        &mut session.types,
        &mut session.generic_instances,
    );

    if let Err(()) = type_solver.check_all_types_infered(
        &session.types,
        &session.generic_instances,
        &session.generic_def_span_rev,
        &dispatched_calls,
    ) {
        has_error = true;
    }

    if has_error {
        // In order to create error messages, we have to convert spans to strings.
        // But that's very expensive operation, so we initialize this map only when there's an error.
        session.init_span_string_map();

        for error in type_solver.errors.iter() {
            session.errors.push(session.type_error_to_general_error(error));
        }
    }

    (session, type_solver)
}

// It's very expensive and should be used only for debugging the compiler.
pub fn dump(session: &mut Session, solver: &Solver) {
    session.init_span_string_map();
    let mut renders = vec![];
    let mut render_span_session = RenderSpanSession::new(&session.intermediate_dir);

    for (type_var, id) in solver.type_vars.iter() {
        let mut id = id.map(
            |id| unintern_string(id, &session.intermediate_dir)
                    .unwrap()
                    .unwrap_or(b"????".to_vec())
        );
        let span;

        let r#type = match type_var {
            Type::Var { def_span, .. } => {
                span = Some(*def_span);

                match session.types.get(def_span) {
                    Some(t) => t.clone(),
                    None => type_var.clone(),
                }
            },
            Type::GenericInstance { call, generic } => {
                span = Some(*call);

                if id.is_none() {
                    id = session.span_to_string(*generic).map(|s| s.into_bytes());
                }

                match session.generic_instances.get(&(*call, *generic)) {
                    Some(t) => t.clone(),
                    None => type_var.clone(),
                }
            },
            _ => unreachable!(),
        };

        let span = span.unwrap();

        let rendered = format!(
            "{}: {}\n{}\n\n",
            String::from_utf8_lossy(&id.unwrap_or(b"????".to_vec())).to_string(),
            session.render_type(&r#type),
            render_spans(
                &[RenderableSpan {
                    span,
                    auxiliary: false,
                    note: None,
                }],
                &RenderSpanOption {
                    max_width: 88,
                    max_height: 10,
                    render_source: true,
                    color: Some(ColorOption {
                        primary: Color::Blue,
                        auxiliary: Color::Blue,
                        info: Color::Green,
                    }),
                    group_delim: None,
                },
                &mut render_span_session,
            ),
        );
        renders.push((span, rendered));
    }

    renders.sort_by_key(|(span, _)| *span);

    for (_, r) in renders.iter() {
        println!("{r}");
    }
}
