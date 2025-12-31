use crate::{Expr, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_number_eval::{eval_number_infix_op, eval_number_prefix_op};
use sodigy_span::SpanDeriveKind;

// It can only evaluate char/number/int/byte.
pub fn eval_const(expr: &Expr, session: &mut Session) -> Result<Expr, ()> {
    match expr {
        Expr::Number { .. } |
        Expr::Char { .. } |
        Expr::Byte { .. } => Ok(expr.clone()),
        Expr::PrefixOp { op, op_span, rhs } => {
            let result_span = op_span.merge(rhs.error_span_wide());
            let rhs = eval_const(rhs, session)?;

            match (op, rhs) {
                (_, Expr::Number { n, .. }) => match eval_number_prefix_op(*op, *op_span, &n) {
                    Ok(n) => Ok(Expr::Number { n, span: result_span.derive(SpanDeriveKind::ConstEval) }),
                    Err(es) => {
                        session.errors.extend(es);
                        Err(())
                    },
                },
                _ => {
                    session.errors.push(Error::todo(89468, "more const eval", *op_span));
                    Err(())
                },
            }
        },
        Expr::InfixOp { op, op_span, lhs, rhs } => {
            let result_span = lhs.error_span_wide().merge(*op_span).merge(rhs.error_span_wide());
            let (lhs, rhs) = match (
                eval_const(lhs, session),
                eval_const(rhs, session),
            ) {
                (Ok(lhs), Ok(rhs)) => (lhs, rhs),
                _ => {
                    return Err(());
                },
            };

            match (lhs, op, rhs) {
                (Expr::Number { n: lhs, .. }, _, Expr::Number { n: rhs, .. }) => match eval_number_infix_op(*op, *op_span, &lhs, &rhs) {
                    Ok(n) => Ok(Expr::Number { n, span: result_span.derive(SpanDeriveKind::ConstEval) }),
                    Err(es) => {
                        session.errors.extend(es);
                        Err(())
                    },
                },
                _ => {
                    session.errors.push(Error::todo(89469, "more const eval", *op_span));
                    Err(())
                },
            }
        },
        _ => {
            session.errors.push(Error {
                kind: ErrorKind::CannotEvaluateConst,
                spans: expr.error_span_wide().simple_error(),
                note: None,
            });
            Err(())
        },
    }
}
