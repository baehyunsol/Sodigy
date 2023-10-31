use super::{Expr, ExprKind, LocalDef, Match, MatchArm, Scope};
use crate::err::HirError;
use crate::names::{NameBindingType, NameOrigin, NameSpace};
use crate::pattern::{lower_ast_local_def, lower_ast_pattern};
use crate::session::HirSession;
use crate::warn::HirWarning;
use sodigy_ast::{self as ast, IdentWithSpan, ValueKind};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

pub fn lower_ast_expr(
    e: &ast::Expr,
    session: &mut HirSession,
    used_names: &mut HashSet<(InternedString, NameOrigin)>,

    // `use z as x.y.z;` -> {'z': ['x', 'y', 'z']}
    // span is later used for error messages
    use_cases: &HashMap<InternedString, (SpanRange, Vec<InternedString>)>,

    name_space: &mut NameSpace,
) -> Result<Expr, ()> {
    let res = match &e.kind {
        ast::ExprKind::Value(v) => match &v {
            ValueKind::Identifier(id) => {
                if let Some(u) = use_cases.get(id) {
                    // unfold u
                    todo!()
                }

                else if let Some(origin) = name_space.find_origin(*id) {
                    used_names.insert((*id, origin));

                    Expr {
                        kind: ExprKind::Identifier(*id, origin),
                        span: e.span,
                    }
                }

                else {
                    session.push_error(HirError::undefined_name(
                        IdentWithSpan::new(*id, e.span),

                        // This is VERY EXPENSIVE
                        // make sure it's called only when the compilation fails
                        name_space.find_similar_names(*id),
                    ));
                    return Err(());
                }
            },
            ValueKind::Number(n) => if n.is_integer() {
                Expr {
                    kind: ExprKind::Integer(*n),
                    span: e.span,
                }
            } else {
                Expr {
                    kind: ExprKind::Ratio(*n),
                    span: e.span,
                }
            },
            ValueKind::String { s, is_binary } => Expr {
                kind: ExprKind::String { s: *s, is_binary: *is_binary },
                span: e.span,
            },
            ValueKind::Char(c) => todo!(),
            ValueKind::List(elems) => todo!(),
            ValueKind::Tuple(elems) => todo!(),
            ValueKind::Format(elems) => todo!(),
            ValueKind::Lambda {
                args, value,
            } => {
                // Push names defined in this lambda, then recurs
                todo!();

                // check unused-names
                todo!();
            },
            ValueKind::Scope { scope, uid } => {
                let mut name_bindings = HashSet::new();
                let mut name_collision_checker = HashMap::new();

                // push name bindings to the name space
                // find name collisions
                for ast::LocalDef { pattern, .. } in scope.defs.iter() {
                    for def in pattern.get_name_bindings().iter() {
                        match name_collision_checker.get(def.id()) {
                            Some(id) => {
                                session.push_error(HirError::name_collision(*def, *id));
                            },
                            None => {
                                name_collision_checker.insert(*def.id(), *def);
                            },
                        }

                        name_bindings.insert(*def.id());
                    }
                }

                name_space.push_locals(
                    *uid,
                    name_bindings,
                );

                // lower defs and values
                let local_defs: Vec<Result<LocalDef, ()>> = scope.defs.iter().map(
                    |local_def| lower_ast_local_def(
                        local_def,
                        session,
                        used_names,
                        use_cases,
                        name_space,
                    )
                ).collect();

                let value = lower_ast_expr(
                    scope.value.as_ref(),
                    session,
                    used_names,
                    use_cases,
                    name_space,
                );

                // find unused names
                for (id, id_with_span) in name_collision_checker.iter() {
                    if !used_names.contains(&(*id, NameOrigin::Local { origin: *uid })) {
                        session.push_warning(HirWarning::unused_name(*id_with_span, NameBindingType::LocalScope));
                    }
                }

                // we have to unwrap errors as late as possible
                // so that we can find as many as possible
                let mut has_error = false;

                let local_defs: Vec<LocalDef> = local_defs.into_iter().filter_map(
                    |d| match d {
                        Ok(d) => Some(d),
                        Err(_) => {
                            has_error = true;
                            None
                        }
                    }
                ).collect();

                name_space.pop_locals();

                let value = value?;

                if has_error {
                    return Err(());
                }

                // very simple optimization: `{ x }` -> `x`
                // TODO: make ALL the optimizations configurable
                if local_defs.is_empty() {
                    value
                }

                else {
                    Expr {
                        kind: ExprKind::Scope(Scope {
                            defs: local_defs,
                            value: Box::new(value),
                            uid: *uid,
                        }),
                        span: e.span,
                    }
                }
            },
        },
        ast::ExprKind::PrefixOp(op, expr) => Expr {
            kind: ExprKind::PrefixOp(
                *op,
                Box::new(lower_ast_expr(
                    expr,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                )?),
            ),
            span: e.span,
        },
        ast::ExprKind::PostfixOp(op, expr) => Expr {
            kind: ExprKind::PostfixOp(
                *op,
                Box::new(lower_ast_expr(
                    expr,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                )?),
            ),
            span: e.span,
        },
        ast::ExprKind::InfixOp(op, lhs, rhs) => {
            // we should unwrap these after both lowerings are complete
            // it helps us find more errors in case there are ones
            let lhs = lower_ast_expr(
                lhs,
                session,
                used_names,
                use_cases,
                name_space,
            );
            let rhs = lower_ast_expr(
                rhs,
                session,
                used_names,
                use_cases,
                name_space,
            );

            Expr {
                kind: ExprKind::InfixOp(
                    *op,
                    Box::new(lhs?),
                    Box::new(rhs?),
                ),
                span: e.span,
            }
        },
        ast::ExprKind::Path { pre, post } => todo!(),
        ast::ExprKind::Call { functor, args } => todo!(),
        ast::ExprKind::StructInit { struct_, init } => todo!(),
        ast::ExprKind::Branch(arms) => {
            // Push names defined in the arms (if there's `if let`), then recurs
            todo!();

            // check unused-names
            todo!();
        },
        ast::ExprKind::Match { value, arms } => {
            let result_value = lower_ast_expr(
                value,
                session,
                used_names,
                use_cases,
                name_space,
            );
            let mut result_arms = Vec::with_capacity(arms.len());

            for ast::MatchArm {
                pattern,
                guard,
                value,
                uid,
            } in arms.iter() {
                // TODO: it's a copy-paste of ast::ExprKind::Scope
                let mut name_bindings = HashSet::new();
                let mut name_collision_checker = HashMap::new();

                for def in pattern.get_name_bindings().iter() {
                    match name_collision_checker.get(def.id()) {
                        Some(id) => {
                            session.push_error(HirError::name_collision(*def, *id));
                        },
                        None => {
                            name_collision_checker.insert(*def.id(), *def);
                        },
                    }

                    name_bindings.insert(*def.id());
                }

                name_space.push_locals(
                    *uid,
                    name_bindings,
                );

                let value = lower_ast_expr(
                    value,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                );

                let pattern = lower_ast_pattern(
                    pattern,
                    session,
                );

                let guard = guard.as_ref().map(|g| lower_ast_expr(
                    g,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                ));

                // find unused names
                for (id, id_with_span) in name_collision_checker.iter() {
                    if !used_names.contains(&(*id, NameOrigin::Local { origin: *uid })) {
                        session.push_warning(HirWarning::unused_name(*id_with_span, NameBindingType::MatchArm));
                    }
                }

                name_space.pop_locals();

                result_arms.push(MatchArm {
                    value: value?,
                    pattern: pattern?,
                    guard: if let Some(g) = guard { Some(g?) } else { None },
                });
            }

            Expr {
                kind: ExprKind::Match(Match {
                    arms: result_arms,
                    value: Box::new(result_value?),
                }),
                span: e.span,
            }
        },
    };

    Ok(res)
}
