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

mod error;
mod solver;

pub use error::{ErrorContext, RenderTypeError, TypeError};
use solver::Solver;

pub fn solve(mut session: Session) -> (Session, Solver) {
    let mut solver = Solver::new(session.lang_items.clone());
    let mut generic_funcs = vec![];

    for func in session.funcs.iter() {
        let _ = solver.solve_func(func, &mut session.types, &mut session.generic_instances);

        if !func.generics.is_empty() {
            generic_funcs.push(func);
        }
    }

    for r#let in session.lets.iter() {
        let _ = solver.solve_let(r#let, &mut session.types, &mut session.generic_instances);
    }

    for assert in session.asserts.iter() {
        let _ = solver.solve_assert(assert, &mut session.types, &mut session.generic_instances);
    }

    if !generic_funcs.is_empty() {
        todo!()
    }

    solver.check_all_types_infered(
        &session.types,
        &session.generic_instances,
        &session.generic_def_span_rev,
    );

    // In order to create error messages, we have to convert spans to strings.
    // But that's very expensive operation, so we initialize this map only when there's an error.
    if !solver.errors.is_empty() {
        session.init_span_string_map();
    }

    for error in solver.errors.iter() {
        session.errors.push(session.type_error_to_general_error(error));
    }

    (session, solver)
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
