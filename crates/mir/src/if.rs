use crate::{Expr, Match, MatchArm, Session, ShortCircuitKind};
use sodigy_hir as hir;
use sodigy_span::{Span, SpanDeriveKind};

// If it's lowered from a short circuit operator,
// `if_span` is `op_span` and `else_span` is None.
#[derive(Clone, Debug)]
pub struct If {
    pub if_span: Span,
    pub cond: Box<Expr>,
    pub else_span: Span,
    pub true_value: Box<Expr>,
    pub true_group_span: Span,
    pub false_value: Box<Expr>,
    pub false_group_span: Span,

    // `&&` and `||` operator are lowered to `if`.
    pub from_short_circuit: Option<ShortCircuitKind>,
}

pub fn lower_hir_if(hir_if: &hir::If, session: &mut Session) -> Result<Expr, ()> {
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

    if let (Some(let_span), Some(pattern)) = (hir_if.let_span, &hir_if.pattern) {
        Ok(Expr::Match(Match {
            keyword_span: hir_if.if_span.merge(let_span).derive(SpanDeriveKind::IfLet),
            scrutinee: Box::new(cond),
            arms: vec![
                MatchArm {
                    pattern: pattern.clone(),
                    guard: None,
                    value: true_value,
                },
                MatchArm {
                    pattern: hir::Pattern {
                        name: None,
                        name_span: None,
                        kind: hir::PatternKind::Wildcard(Span::None),
                    },
                    guard: None,
                    value: false_value,
                },
            ],
            group_span: Span::None,
            lowered_from_if: true,
        }))
    }

    else {
        Ok(Expr::If(If {
            if_span: hir_if.if_span,
            cond: Box::new(cond),
            else_span: hir_if.else_span,
            true_value: Box::new(true_value),
            true_group_span: hir_if.true_group_span,
            false_value: Box::new(false_value),
            false_group_span: hir_if.false_group_span,
            from_short_circuit: None,
        }))
    }
}
