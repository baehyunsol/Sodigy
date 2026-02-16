use sodigy_mir::{Callable, Expr, Session as MirSession};

mod error;
mod field;
mod r#match;
mod session;

pub use error::PatternAnalysisError;
pub(crate) use field::lower_fields;
pub(crate) use r#match::lower_match;
use session::Session;

pub fn lower(mir_session: &mut MirSession) -> Result<(), ()> {
    let mut has_error = false;
    let mut session = Session::from_mir_session(mir_session);

    for r#let in mir_session.lets.iter_mut() {
        has_error |= lower_expr(&mut r#let.value, &mut session).is_err();
    }

    for func in mir_session.funcs.iter_mut() {
        has_error |= lower_expr(&mut func.value, &mut session).is_err();
    }

    for assert in mir_session.asserts.iter_mut() {
        if let Some(note) = &mut assert.note {
            has_error |= lower_expr(note, &mut session).is_err();
        }

        has_error |= lower_expr(&mut assert.value, &mut session).is_err();
    }

    mir_session.errors.extend(session.errors.drain(..));
    mir_session.warnings.extend(session.warnings.drain(..));

    if has_error {
        Err(())
    }

    else {
        Ok(())
    }
}

fn lower_expr(expr: &mut Expr, session: &mut Session) -> Result<(), ()> {
    match expr {
        Expr::Ident(_) | Expr::Constant(_) => Ok(()),
        Expr::If(r#if) => match (
            lower_expr(r#if.cond.as_mut(), session),
            lower_expr(r#if.true_value.as_mut(), session),
            lower_expr(r#if.false_value.as_mut(), session),
        ) {
            (Ok(()), Ok(()), Ok(())) => Ok(()),
            _ => Err(()),
        },
        Expr::Block(block) => {
            let mut has_error = false;

            for r#let in block.lets.iter_mut() {
                has_error |= lower_expr(&mut r#let.value, session).is_err();
            }

            for assert in block.asserts.iter_mut() {
                if let Some(note) = &mut assert.note {
                    has_error |= lower_expr(note, session).is_err();
                }

                has_error |= lower_expr(&mut assert.value, session).is_err();
            }

            has_error |= lower_expr(&mut block.value, session).is_err();

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
        Expr::Field { lhs, fields } => {
            lower_fields(lhs, fields, session);
            lower_expr(lhs, session)
        },
        Expr::FieldUpdate { lhs, rhs, fields } => {
            lower_fields(lhs, fields, session);

            let lhs_err = lower_expr(lhs, session).is_err();
            let rhs_err = lower_expr(rhs, session).is_err();

            if lhs_err || rhs_err {
                Err(())
            }

            else {
                Ok(())
            }
        },
        Expr::Match(r#match) => {
            let mut has_error = false;

            has_error |= lower_expr(&mut r#match.scrutinee, session).is_err();

            for arm in r#match.arms.iter_mut() {
                if let Some(guard) = &mut arm.guard {
                    has_error |= lower_expr(guard, session).is_err();
                }

                has_error |= lower_expr(&mut arm.value, session).is_err();
            }

            match lower_match(r#match, session) {
                Ok(lowered) => {
                    *expr = lowered;
                },
                Err(()) => {
                    has_error = true;
                },
            }

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
        Expr::Call { func, args, .. } => {
            let mut has_error = false;

            match func {
                Callable::Dynamic(f) => {
                    if let Err(()) = lower_expr(f, session) {
                        has_error = true;
                    }
                },
                _ => {},
            }

            for arg in args.iter_mut() {
                if let Err(()) = lower_expr(arg, session) {
                    has_error = true;
                }
            }

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
    }
}
