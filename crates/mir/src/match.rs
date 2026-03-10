// `hir::Match` is first lowered to `mir::Match`. They're almost identical, except
// that `hir::Expr`s are lowered to `mir::Expr`s.
//
// After type-checking, we can lower `mir::Match` to `mir::MatchFsm`, which knows
// how to destructure patterns. It also does exhaustiveness checking.

use crate::{Expr, Pattern, Session};
use sodigy_hir::{self as hir, PatternSplit};
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Match {
    pub keyword_span: Span,
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub group_span: Span,
    pub lowered_from_if: bool,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,

    // If a pattern in the arm has or-patterns, the arm might be splitted into multiple arms.
    // In that case, the splitted arms are assigned the same group id.
    // Group ids are unique inside a match expression.
    pub group_id: Option<u32>,
}

pub enum ArmSplit<'a> {
    NoSplit(&'a MatchArm),
    Split(Vec<MatchArm>),
}

impl Match {
    pub fn from_hir(hir_match: &hir::Match, session: &mut Session) -> Result<Match, ()> {
        let mut has_error = false;
        let scrutinee = match Expr::from_hir(&hir_match.scrutinee, session) {
            Ok(scrutinee) => Some(scrutinee),
            Err(()) => {
                has_error = true;
                None
            },
        };
        let mut arms = Vec::with_capacity(hir_match.arms.len());

        for hir_arm in hir_match.arms.iter() {
            match MatchArm::from_hir(hir_arm, session) {
                Ok(arm) => {
                    arms.push(arm);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Match {
                keyword_span: hir_match.keyword_span,
                scrutinee: Box::new(scrutinee.unwrap()),
                arms,
                group_span: hir_match.group_span,
                lowered_from_if: false,
            })
        }
    }
}

impl MatchArm {
    pub fn from_hir(hir_arm: &hir::MatchArm, session: &mut Session) -> Result<MatchArm, ()> {
        match (hir_arm.guard.as_ref().map(|guard| Expr::from_hir(guard, session)), Expr::from_hir(&hir_arm.value, session)) {
            (Some(Ok(guard)), Ok(value)) => Ok(MatchArm {
                pattern: hir_arm.pattern.clone(),
                guard: Some(guard),
                value,
                group_id: None,
            }),
            (None, Ok(value)) => Ok(MatchArm {
                pattern: hir_arm.pattern.clone(),
                guard: None,
                value,
                group_id: None,
            }),
            _ => Err(()),
        }
    }

    pub fn split_or_patterns<'s>(&'s self) -> ArmSplit<'s> {
        match self.pattern.split_or_patterns() {
            PatternSplit::NoSplit(_) => ArmSplit::NoSplit(self),
            PatternSplit::Split(patterns) => {
                let mut arms = Vec::with_capacity(patterns.len());

                // We don't derive spans here. If we do so, we also have to
                // derive some def_spans (for name bindings from patterns),
                // but it's really difficult to do so.
                for (pattern, split_id) in patterns.into_iter() {
                    arms.push(MatchArm {
                        pattern,
                        guard: self.guard.clone(),
                        value: self.value.clone(),
                        group_id: Some(split_id),
                    });
                }

                ArmSplit::Split(arms)
            },
        }
    }
}
