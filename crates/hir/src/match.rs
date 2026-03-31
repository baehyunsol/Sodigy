use crate::{Expr, Pattern, Session};
use sodigy_name_analysis::{
    NameKind,
    Namespace,
    UseCount,
};
use sodigy_parse as ast;
use sodigy_span::{Span, SpanDeriveKind};
use sodigy_token::InfixOp;

#[derive(Clone, Debug)]
pub struct Match {
    pub keyword_span: Span,
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub group_span: Span,
    pub lowered_from_let: bool,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,
}

impl Match {
    pub fn from_ast(ast_match: &ast::Match, session: &mut Session) -> Result<Match, ()> {
        let mut has_error = false;
        let mut arms = vec![];

        let scrutinee = match Expr::from_ast(ast_match.scrutinee.as_ref(), session) {
            Ok(scrutinee) => Some(scrutinee),
            Err(()) => {
                has_error = true;
                None
            },
        };

        for ast_arm in ast_match.arms.iter() {
            let mut extra_guards = vec![];
            let pattern = match Pattern::from_ast(&ast_arm.pattern, session, &mut extra_guards) {
                Ok(pattern) => Some(pattern),
                Err(()) => {
                    has_error = true;
                    None
                },
            };
            let names = ast_arm.pattern.bound_names().iter().map(
                |(id, span)| (*id, (span.clone(), NameKind::PatternNameBind, UseCount::new()))
            ).collect();

            session.name_stack.push(Namespace::Pattern { names });

            let mut guard = match ast_arm.guard.as_ref().map(|guard| Expr::from_ast(guard, session)) {
                Some(Ok(guard)) => Some(guard),
                Some(Err(())) => {
                    has_error = true;
                    None
                },
                None => None,
            };

            if !extra_guards.is_empty() {
                let extra_guards: Vec<Expr> = extra_guards.into_iter().map(
                    |guard| guard.condition
                ).collect();
                let tmp_span = extra_guards[0].error_span_wide().derive(SpanDeriveKind::ExprInPattern);
                let mut extra_guard = fold_exprs(
                    extra_guards,
                    InfixOp::LogicAnd,
                    tmp_span,
                );

                match guard.take() {
                    Some(g) => {
                        let tmp_span = g.error_span_wide().derive(SpanDeriveKind::ExprInPattern);
                        extra_guard = fold_exprs(vec![extra_guard, g], InfixOp::LogicAnd, tmp_span);
                        guard = Some(extra_guard);
                    },
                    None => {
                        guard = Some(extra_guard);
                    },
                }
            }

            let value = match Expr::from_ast(&ast_arm.value, session) {
                Ok(value) => Some(value),
                Err(()) => {
                    has_error = true;
                    None
                },
            };

            let Some(Namespace::Pattern { names }) = session.name_stack.pop() else { unreachable!() };
            session.warn_unused_names(&names);

            if !has_error {
                arms.push(MatchArm {
                    pattern: pattern.unwrap(),
                    guard,
                    value: value.unwrap(),
                });
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Match {
                keyword_span: ast_match.keyword_span.clone(),
                scrutinee: Box::new(scrutinee.unwrap()),
                arms,
                group_span: ast_match.group_span.clone(),
                lowered_from_let: ast_match.lowered_from_let,
            })
        }
    }
}

pub fn fold_exprs(
    mut exprs: Vec<Expr>,
    op: InfixOp,
    op_span: Span,
) -> Expr {
    match (exprs.pop(), exprs.len()) {
        (None, _) => unreachable!(),
        (Some(e), 0) => e,
        (Some(e), _) => Expr::InfixOp {
            op,
            op_span: op_span.clone(),
            lhs: Box::new(e),
            rhs: Box::new(fold_exprs(exprs, op, op_span)),
        },
    }
}
