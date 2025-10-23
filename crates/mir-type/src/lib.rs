pub(crate) use sodigy_mir::{Expr, Type};
use sodigy_mir::Session;

mod error;
mod preludes;
mod solver;

pub use error::{ErrorContext, RenderTypeError, TypeError, TypeErrorKind};
use solver::Solver;

pub fn solve(mut session: Session) -> Session {
    let mut solver = Solver::new();

    for func in session.funcs.iter() {
        let _ = solver.solve_func(func, &mut session.types);
    }

    for r#let in session.lets.iter() {
        let _ = solver.solve_let(r#let, &mut session.types);
    }

    for assert in session.asserts.iter() {
        let _ = solver.solve_assert(assert, &mut session.types);
    }

    solver.check_all_types_infered(&session.types);

    // In order to create error messages, we have to convert spans to strings.
    // But that's very expensive operation, so we initialize this map only when there's an error.
    if !solver.errors.is_empty() {
        session.init_span_string_map();
    }

    for error in solver.errors.iter() {
        session.errors.push(session.type_error_to_general_error(error));
    }

    session
}
