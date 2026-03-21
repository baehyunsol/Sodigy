use crate::field::lower_fields;
use crate::r#match::lower_match;
use sodigy_mir::{Callable, Expr, Session as MirSession};

mod field;
mod r#match;
mod session;

pub use r#match::MatchDump;
pub use session::Session;

pub fn lower<'a, 'b>(mir_session: &mut MirSession<'a, 'b>, dump_matches: bool) -> Session<'a, 'b> {
    let mut session = Session::from_mir_session(mir_session, dump_matches);

    for r#let in mir_session.lets.iter_mut() {
        let _ = lower_expr(&mut r#let.value, &mut session);
    }

    for func in mir_session.funcs.iter_mut() {
        let _ = lower_expr(&mut func.value, &mut session);
    }

    for assert in mir_session.asserts.iter_mut() {
        if let Some(note) = &mut assert.note {
            let _ = lower_expr(note, &mut session);
        }

        let _ = lower_expr(&mut assert.value, &mut session);
    }

    mir_session.errors.extend(session.errors.drain(..));
    mir_session.warnings.extend(session.warnings.drain(..));
    session
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
            // It must call `lower_match` before `lower_expr` because
            // `lower_match` creates new field expressions.
            let lowered = lower_match(r#match, session)?;
            *expr = lowered;
            lower_expr(expr, session)
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
