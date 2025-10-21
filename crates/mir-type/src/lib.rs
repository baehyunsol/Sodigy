use sodigy_mir::{Session, Type};

mod error;
mod preludes;
mod solver;

pub(crate) use error::TypeError;
use solver::{Solver, TypeSolve};

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
    let mut has_error = false;

    for func in session.funcs.iter() {
        let infered_type = match solver.infer_expr(&func.value, &mut session.types) {
            Ok(r#type) => r#type,

            // convert `e` to `Error` and break
            // we should not do any more inference if there's an error
            Err(e) => todo!(),
        };

        // TODO: It's always `Some(_)` because I just inserted it
        if let Some(annotated_type) = session.types.get(&func.name_span) {
            if infered_type.has_variable() {
                // deal with the borrow checker
                let annotated_type = annotated_type.clone();

                if let Err(e) = solver.unify(
                    &infered_type,
                    &annotated_type,
                    &mut session.types,
                ) {
                    // convert `e` to `Error` and break
                    // we should not do any more inference if there's an error
                    todo!()
                }
            }
        }

        else {
            if let Err(e) = solver.unify(
                &Type::Var(func.name_span),
                &infered_type,
                &mut session.types,
            ) {
                // convert `e` to `Error` and break
                // we should not do any more inference if there's an error
                todo!()
            }
        }
    }

    if !has_error {
        for r#let in session.lets.iter() {
            let infered_type = match solver.infer_expr(&r#let.value, &mut session.types) {
                Ok(r#type) => r#type,

                // convert `e` to `Error` and break
                // we should not do any more inference if there's an error
                Err(e) => todo!(),
            };

            // TODO: It's always `Some(_)` because I just inserted it
            if let Some(annotated_type) = session.types.get(&r#let.name_span) {
                if infered_type.has_variable() {
                    // deal with the borrow checker
                    let annotated_type = annotated_type.clone();

                    if let Err(e) = solver.unify(
                        &infered_type,
                        &annotated_type,
                        &mut session.types,
                    ) {
                        // convert `e` to `Error` and break
                        // we should not do any more inference if there's an error
                        todo!()
                    }
                }
            }

            else {
                if let Err(e) = solver.unify(
                    &Type::Var(r#let.name_span),
                    &infered_type,
                    &mut session.types,
                ) {
                    // convert `e` to `Error` and break
                    // we should not do any more inference if there's an error
                    todo!()
                }
            }
        }
    }

    session
}
