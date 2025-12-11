// `hir::Match` is first lowered to `mir::Match`. They're almost identical, except
// that `hir::Expr`s are lowered to `mir::Expr`s.
//
// After type-checking, we can lower `mir::Match` to `mir::MatchFsm`, which knows
// how to destructure patterns. It also does exhaustiveness checking.

use crate::{Callable, Expr, Session, Type, type_of};
use sodigy_error::{Error, Warning};
use sodigy_hir::{self as hir, Generic, StructField};
use sodigy_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Match {
    pub keyword_span: Span,
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub lowered_from_if: bool,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: hir::Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,
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
            }),
            (None, Ok(value)) => Ok(MatchArm {
                pattern: hir_arm.pattern.clone(),
                guard: None,
                value,
            }),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatchFsm {}  // TODO
