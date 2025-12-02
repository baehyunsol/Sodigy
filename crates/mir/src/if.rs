use crate::{Expr, Pattern, Session, ShortCircuitKind};
use sodigy_hir as hir;
use sodigy_span::Span;

// If it's lowered from a short circuit operator,
// `if_span` is `op_span` and `else_span` is None.
#[derive(Clone, Debug)]
pub struct If {
    pub if_span: Span,
    pub cond: Box<Expr>,
    pub pattern: Option<Pattern>,
    pub else_span: Span,
    pub true_value: Box<Expr>,
    pub false_value: Box<Expr>,

    // `&&` and `||` operator are lowered to `if`.
    pub from_short_circuit: Option<ShortCircuitKind>,
}

impl If {
    pub fn from_hir(hir_if: &hir::If, session: &mut Session) -> Result<If, ()> {
        let (cond, true_value, false_value) = match (
            Expr::from_hir(hir_if.cond.as_ref(), session),
            Expr::from_hir(hir_if.true_value.as_ref(), session),
            Expr::from_hir(hir_if.false_value.as_ref(), session),
        ) {
            (Ok(cond), Ok(true_value), Ok(false_value)) => (cond, true_value, false_value),
            _ => {
                return Err(());
            },
        };

        if let Some(pattern) = &hir_if.pattern {
            todo!()
        }

        Ok(If {
            if_span: hir_if.if_span,
            cond: Box::new(cond),
            pattern: None,  // TODO
            else_span: hir_if.else_span,
            true_value: Box::new(true_value),
            false_value: Box::new(false_value),
            from_short_circuit: None,
        })
    }
}
