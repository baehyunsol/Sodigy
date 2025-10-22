pub(crate) use sodigy_mir::{Expr, Type};
use sodigy_mir::Session;

mod error;
mod preludes;
mod solver;

pub use error::{ErrorContext, TypeError, TypeErrorKind};
use solver::Solver;

pub fn solve(mut session: Session) -> Session {
    let mut solver = Solver::new();

    for func in session.funcs.iter() {
        let infered_type = match solver.solve_expr(&func.value, &mut session.types) {
            Ok(r#type) => r#type,
            Err(()) => {
                continue;
            },
        };

        let (
            annotated_type,
            error_span,
            extra_error_span,
            context,
        ) = match session.types.get(&func.name_span) {
            None | Some(Type::Var(_)) => (
                Type::Var(func.name_span),
                func.value.error_span(),
                None,
                ErrorContext::InferTypeAnnotation,
            ),
            Some(annotated_type) => (
                annotated_type.clone(),
                func.value.error_span(),
                func.type_annotation_span,
                ErrorContext::VerifyTypeAnnotation,
            ),
        };

        let _ = solver.equal(
            &annotated_type,
            &infered_type,
            &mut session.types,
            error_span,
            extra_error_span,
            context,
        );
    }

    for r#let in session.lets.iter() {
        let infered_type = match solver.solve_expr(&r#let.value, &mut session.types) {
            Ok(r#type) => r#type,
            Err(()) => {
                continue;
            },
        };

        let (
            annotated_type,
            error_span,
            extra_error_span,
            context,
        ) = match session.types.get(&r#let.name_span) {
            None | Some(Type::Var(_)) => (
                Type::Var(r#let.name_span),
                r#let.value.error_span(),
                None,
                ErrorContext::InferTypeAnnotation,
            ),
            Some(annotated_type) => (
                annotated_type.clone(),
                r#let.value.error_span(),
                r#let.type_annotation_span,
                ErrorContext::VerifyTypeAnnotation,
            ),
        };

        let _ = solver.equal(
            &annotated_type,
            &infered_type,
            &mut session.types,
            error_span,
            extra_error_span,
            context,
        );
    }

    for error in solver.errors.into_iter() {
        todo!();
    }

    session
}
