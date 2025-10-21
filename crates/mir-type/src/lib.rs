use sodigy_mir::{Session, Type};

mod error;
mod preludes;
mod solver;

pub(crate) use error::TypeError;
use solver::Solver;

pub fn infer_and_check(mut session: Session) -> Session {
    for func in session.funcs.iter() {
        if session.types.get(&func.name_span).is_none() {
            session.types.insert(func.name_span, Type::Var(func.name_span));
        }

        for arg in func.args.iter() {
            if session.types.get(&arg.name_span).is_none() {
                session.types.insert(arg.name_span, Type::Var(arg.name_span));
            }
        }
    }

    for r#let in session.lets.iter() {
        if session.types.get(&r#let.name_span).is_none() {
            session.types.insert(r#let.name_span, Type::Var(r#let.name_span));
        }
    }

    let mut solver = Solver::new();

    for func in session.funcs.iter() {
        solver.infer_expr(&func.value, &mut session.types);
    }

    session
}
