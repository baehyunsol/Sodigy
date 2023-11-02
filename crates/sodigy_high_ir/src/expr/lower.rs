use super::{Expr, ExprKind, Lambda, LocalDef, Match, MatchArm, Scope};
use crate::lower_ast_ty;
use crate::err::HirError;
use crate::func::Arg;
use crate::names::{IdentWithOrigin, NameBindingType, NameOrigin, NameSpace};
use crate::pattern::{lower_ast_local_def, lower_ast_pattern};
use crate::session::HirSession;
use crate::warn::HirWarning;
use sodigy_ast::{self as ast, IdentWithSpan, ValueKind};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

pub fn lower_ast_expr(
    e: &ast::Expr,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,

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
                    used_names.insert(IdentWithOrigin::new(*id, origin));

                    Expr {
                        kind: ExprKind::Identifier(IdentWithOrigin::new(*id, origin)),
                        span: e.span,
                    }
                }

                else {
                    // `def foo(x: Int, y: x)`: no dependent types
                    if name_space.func_args_locked && name_space.is_func_arg_name(id) {
                        session.push_error(HirError::no_dependent_types(
                            IdentWithSpan::new(*id, e.span),
                        ));
                    }

                    else {
                        session.push_error(HirError::undefined_name(
                            IdentWithSpan::new(*id, e.span),
    
                            // This is VERY EXPENSIVE
                            // make sure it's called only when the compilation fails
                            name_space.find_similar_names(*id),
                        ));
                    }

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
            v @ (ValueKind::List(elems)
            | ValueKind::Tuple(elems)) => {
                let is_list = matches!(v, ValueKind::List(_));
                let mut hir_elems = Vec::with_capacity(elems.len());
                let mut has_error = false;

                for elem in elems.iter() {
                    if let Ok(elem) = lower_ast_expr(
                        elem,
                        session,
                        used_names,
                        use_cases,
                        name_space,
                    ) {
                        hir_elems.push(elem);
                    }

                    else {
                        has_error = true;
                    }
                }

                if has_error {
                    return Err(());
                }

                Expr {
                    kind: if is_list { ExprKind::List(hir_elems) } else { ExprKind::Tuple(hir_elems) },
                    span: e.span,
                }
            },
            ValueKind::Format(elems) => {
                // remove empty strings
                // unwrap the entire f-string if there's no value
                // concat consecutive strings

                todo!()
            },
            ValueKind::Lambda {
                args, value, uid,
            } => {
                let mut hir_args = Vec::with_capacity(args.len());
                let mut arg_names = HashMap::with_capacity(args.len());
                let mut foreign_names = vec![];
                let mut has_error = false;

                for ast::ArgDef { name, ty, has_question_mark } in args.iter() {
                    match arg_names.insert(*name.id(), *name) {
                        Some(prev) => {
                            session.push_error(HirError::name_collision(prev, *name));
                            has_error = true;
                        },
                        _ => {},
                    }

                    // TODO: what if `ty` has foreign names?
                    // e.g: `def foo<T>() = \{x: T, x};`
                    // e.g: `def foo(t: Type) = \{x:t, x};`
                    let ty = if let Some(ty) = ty {
                        if let Ok(mut ty) = lower_ast_ty(
                            ty,
                            session,
                            used_names,
                            use_cases,
                            name_space,
                        ) {
                            find_and_replace_foreign_names(
                                &mut ty.0,
                                *uid,
                                &mut foreign_names,
                                used_names,
                                name_space,
                            );

                            Some(ty)
                        } else {
                            has_error = true;
                            None
                        }
                    } else {
                        None
                    };

                    hir_args.push(Arg {
                        name: *name,
                        ty,
                        has_question_mark: *has_question_mark,
                    });
                }

                name_space.push_locals(*uid, arg_names.keys().map(|k| *k).collect());

                let value = lower_ast_expr(
                    value,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                );

                name_space.pop_locals();

                let mut value = value?;

                find_and_replace_foreign_names(
                    &mut value,
                    *uid,
                    &mut foreign_names,
                    used_names,
                    name_space,
                );

                // find unused names
                for (id, id_with_span) in arg_names.iter() {
                    if !used_names.contains(&IdentWithOrigin::new(*id, NameOrigin::Local { origin: *uid })) {
                        session.push_warning(HirWarning::unused_name(*id_with_span, NameBindingType::LambdaArg));
                    }
                }

                if has_error {
                    return Err(());
                }

                Expr {
                    kind: ExprKind::Lambda(Lambda {
                        args: hir_args,
                        value: Box::new(value),
                        foreign_names,
                        uid: *uid,
                    }),
                    span: e.span,
                }
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
                    if !used_names.contains(&IdentWithOrigin::new(*id, NameOrigin::Local { origin: *uid })) {
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
        ast::ExprKind::Call { func, args } => {
            let func = lower_ast_expr(
                func,
                session,
                used_names,
                use_cases,
                name_space,
            );

            let mut has_error = false;
            let mut hir_args = Vec::with_capacity(args.len());

            for arg in args.iter() {
                if let Ok(arg) = lower_ast_expr(
                    arg,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                ) {
                    hir_args.push(arg);
                }

                else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            Expr {
                kind: ExprKind::Call { func: Box::new(func?), args: hir_args, },
                span: e.span,
            }
        },
        ast::ExprKind::StructInit { struct_, init } => todo!(),
        ast::ExprKind::Branch(arms) => {
            // TODO: Push names defined in the arms (if there's `if let`), then recurs
            // TODO: check unused-names

            session.push_error(HirError::todo("branch", e.span));
            return Err(());
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
                    if !used_names.contains(&IdentWithOrigin::new(*id, NameOrigin::Local { origin: *uid })) {
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

fn find_and_replace_foreign_names(
    ex: &mut Expr,
    lambda_uid: Uid,
    foreign_names: &mut Vec<IdentWithOrigin>,
    used_names: &mut HashSet<IdentWithOrigin>,
    name_space: &mut NameSpace,
) {
    match &mut ex.kind {
        ExprKind::Integer(_)
        | ExprKind::Ratio(_)
        | ExprKind::String { .. } => { return; },
        ExprKind::Identifier(id_ori) => {
            let origin = *id_ori.origin();

            // checks whether this id is foreign or not
            match origin {
                NameOrigin::Prelude
                | NameOrigin::Global => {
                    /* not foreign */
                    return;
                },
                NameOrigin::FuncArg { .. }
                | NameOrigin::FuncGeneric { .. } => {
                    /* must be foreign */
                },
                NameOrigin::Local { origin: local_origin } => {
                    // has to see whether it's foreign or not
                    // there are 2 cases: Lambda in a Scope, Scope in a Lambda
                    // first case: that's a foreign name and the scope is still in the name_space
                    // second case: that's not a foreign name and we can ignore that
                    if !name_space.has_this_local_uid(local_origin) {
                        return;
                    }
                },

                NameOrigin::Captured { .. } => todo!(),
            }

            let id = *id_ori.id();
            let mut name_index = None;

            // linear search is fine because `foreign_names` is small enough in most cases
            for (ind, id_ori_) in foreign_names.iter().enumerate() {
                let id_ = *id_ori_.id();
                let origin_ = *id_ori_.origin();

                if (id, origin) == (id_, origin_) {
                    name_index = Some(ind);
                    break;
                }
            }

            if name_index == None {
                name_index = Some(foreign_names.len());
                foreign_names.push(IdentWithOrigin::new(id, origin));
            }

            let name_index = name_index.unwrap();
            id_ori.set_origin(NameOrigin::Captured {
                lambda: lambda_uid,
                index: name_index,
            });
            used_names.insert(id_ori.clone());
        },
        ExprKind::Call { func, args } => {
            find_and_replace_foreign_names(
                func,
                lambda_uid,
                foreign_names,
                used_names,
                name_space,
            );

            for arg in args.iter_mut() {
                find_and_replace_foreign_names(
                    arg,
                    lambda_uid,
                    foreign_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::List(elems)
        | ExprKind::Tuple(elems)
        | ExprKind::Format(elems) => {
            for elem in elems.iter_mut() {
                find_and_replace_foreign_names(
                    elem,
                    lambda_uid,
                    foreign_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::Scope(Scope { defs, value, .. }) => {
            find_and_replace_foreign_names(
                value,
                lambda_uid,
                foreign_names,
                used_names,
                name_space,
            );

            // TODO: do we have to look into patterns?
            for LocalDef { value, .. } in defs.iter_mut() {
                find_and_replace_foreign_names(
                    value,
                    lambda_uid,
                    foreign_names,
                    used_names,
                    name_space,
                );
            }
        },
        // TODO: lambda A in lambda B
        // let's say A captures name x
        // case 1: B also captures x
        // case 2: x is defined in B
        ExprKind::Lambda(Lambda {
            args,
            value,
            foreign_names,
            ..
        }) => todo!(),
        ExprKind::Match(Match {
            arms,
            value,
        }) => {
            find_and_replace_foreign_names(
                value,
                lambda_uid,
                foreign_names,
                used_names,
                name_space,
            );

            // TODO: handle patterns and guards
            for MatchArm { value, .. } in arms.iter_mut() {
                find_and_replace_foreign_names(
                    value,
                    lambda_uid,
                    foreign_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::PrefixOp(_, value)
        | ExprKind::PostfixOp(_, value) => {
            find_and_replace_foreign_names(
                value,
                lambda_uid,
                foreign_names,
                used_names,
                name_space,
            );
        },
        ExprKind::InfixOp(_, lhs, rhs) => {
            find_and_replace_foreign_names(
                lhs,
                lambda_uid,
                foreign_names,
                used_names,
                name_space,
            );
            find_and_replace_foreign_names(
                rhs,
                lambda_uid,
                foreign_names,
                used_names,
                name_space,
            );
        },
    }
}
