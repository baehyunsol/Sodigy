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

impl Session {
    pub fn lower_matches(&mut self) -> Result<(), ()> {
        let mut has_error = false;

        for r#let in self.lets.iter_mut() {
            has_error |= lower_matches(
                &mut r#let.value,
                &self.types,
                &self.struct_shapes,
                &self.lang_items,
                &mut self.errors,
                &mut self.warnings,
            ).is_err();
        }

        for func in self.funcs.iter_mut() {
            has_error |= lower_matches(
                &mut func.value,
                &self.types,
                &self.struct_shapes,
                &self.lang_items,
                &mut self.errors,
                &mut self.warnings,
            ).is_err();
        }

        for assert in self.asserts.iter_mut() {
            if let Some(note) = &mut assert.note {
                has_error |= lower_matches(
                    note,
                    &self.types,
                    &self.struct_shapes,
                    &self.lang_items,
                    &mut self.errors,
                    &mut self.warnings,
                ).is_err();
            }

            has_error |= lower_matches(
                &mut assert.value,
                &self.types,
                &self.struct_shapes,
                &self.lang_items,
                &mut self.errors,
                &mut self.warnings,
            ).is_err();
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }
}

fn lower_matches(
    expr: &mut Expr,
    types: &HashMap<Span, Type>,
    struct_shapes: &HashMap<Span, (Vec<StructField>, Vec<Generic>)>,
    lang_items: &HashMap<String, Span>,
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Result<(), ()> {
    match expr {
        Expr::Identifier(_) |
        Expr::Number { .. } |
        Expr::String { .. } |
        Expr::Char { .. } |
        Expr::Byte { .. } => Ok(()),
        Expr::If(r#if) => match (
            lower_matches(r#if.cond.as_mut(), types, struct_shapes, lang_items, errors, warnings),
            lower_matches(r#if.true_value.as_mut(), types, struct_shapes, lang_items, errors, warnings),
            lower_matches(r#if.false_value.as_mut(), types, struct_shapes, lang_items, errors, warnings),
        ) {
            (Ok(()), Ok(()), Ok(())) => Ok(()),
            _ => Err(()),
        },
        Expr::Block(block) => {
            let mut has_error = false;

            for r#let in block.lets.iter_mut() {
                has_error |= lower_matches(
                    &mut r#let.value,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ).is_err();
            }

            for assert in block.asserts.iter_mut() {
                if let Some(note) = &mut assert.note {
                    has_error |= lower_matches(
                        note,
                        types,
                        struct_shapes,
                        lang_items,
                        errors,
                        warnings,
                    ).is_err();
                }

                has_error |= lower_matches(
                    &mut assert.value,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ).is_err();
            }

            has_error |= lower_matches(
                &mut block.value,
                types,
                struct_shapes,
                lang_items,
                errors,
                warnings,
            ).is_err();

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
        Expr::Match(r#match) => {
            let scrutinee_type = type_of(
                &r#match.scrutinee,
                types,
                struct_shapes,
                lang_items,
            ).expect("Internal Compiler Error: Type-check is complete, but it failed to solve an expression!");
            panic!("TODO: scrutinee_type: {scrutinee_type:?}")
        },
        Expr::MatchFsm(_) => unreachable!(),
        Expr::Call { func, args, .. } => {
            let mut has_error = false;

            match func {
                Callable::Dynamic(f) => {
                    if let Err(()) = lower_matches(
                        f,
                        types,
                        struct_shapes,
                        lang_items,
                        errors,
                        warnings,
                    ) {
                        has_error = true;
                    }
                },
                _ => {},
            }

            for arg in args.iter_mut() {
                if let Err(()) = lower_matches(
                    arg,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ) {
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
        _ => panic!("TODO: {expr:?}"),
    }
}
