use super::{
    Branch,
    BranchArm,
    Expr,
    ExprKind,
    Lambda,
    Match,
    MatchArm,
    Scope,
    ScopedLet,
    StructInit,
    StructInitField,
    lambda::{
        find_and_replace_captured_values,
        ValueCaptureCtxt,
    },
};
use crate::lower_ast_ty;
use crate::attr::lower_ast_attributes;
use crate::error::HirError;
use crate::func::Arg;
use crate::names::{IdentWithOrigin, NameBindingType, NameOrigin, NameSpace};
use crate::pattern::{DestructuredPattern, lower_patterns_to_name_bindings, lower_ast_pattern};
use crate::session::HirSession;
use crate::walker::mut_walker_expr;
use crate::warn::HirWarning;
use sodigy_ast::{self as ast, FieldKind, IdentWithSpan, ValueKind};
use sodigy_intern::InternedString;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

// This function tries to continue lowering even when errors are found.
// Further lowering can find more errors, which is helpful for users.
pub fn lower_ast_expr(
    e: &ast::Expr,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,

    // `import x.y.z as z;` -> {'z': ['x', 'y', 'z']}
    // span is later used for error messages
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,

    name_space: &mut NameSpace,
) -> Result<Expr, ()> {
    let res = match &e.kind {
        ast::ExprKind::Value(v) => match &v {
            ValueKind::Identifier(id) => {
                if let Some((span, names)) = imports.get(id) {
                    if names.len() == 1 {
                        Expr {
                            kind: ExprKind::Identifier(IdentWithOrigin::new(
                                names[0].id(), NameOrigin::Global { origin: None },
                            )),

                            // it points to the `import` statement
                            span: *span,
                        }
                    }

                    else {
                        fields_from_vec(names, *span)
                    }
                }

                else if let Some(origin) = name_space.find_origin(*id) {
                    used_names.insert(IdentWithOrigin::new(*id, origin));

                    Expr {
                        kind: ExprKind::Identifier(IdentWithOrigin::new(*id, origin)),
                        span: e.span,
                    }
                }

                else {
                    // `let foo(x: Int, y: x)`: no dependent types
                    if name_space.func_args_locked && name_space.is_func_arg_name(*id) {
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
            ValueKind::String { content, is_binary } => Expr {
                kind: ExprKind::String { content: *content, is_binary: *is_binary },
                span: e.span,
            },
            ValueKind::Char(c) => Expr {
                kind: ExprKind::Char(*c),
                span: e.span,
            },
            v @ (ValueKind::List(elems)
            | ValueKind::Tuple(elems)) => {
                let is_list = matches!(v, ValueKind::List(_));
                let mut hir_elems = Vec::with_capacity(elems.len());
                let mut has_error = false;

                for elem in elems.iter() {
                    try_warn_unnecessary_paren(elem, session);

                    if let Ok(elem) = lower_ast_expr(
                        elem,
                        session,
                        used_names,
                        imports,
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
                let mut result = Vec::with_capacity(elems.len());
                let mut has_error = false;

                for elem in elems.iter() {
                    match &elem.kind {
                        ast::ExprKind::Value(ast::ValueKind::String {
                            content,
                            is_binary: false,
                        }) if content.is_empty() => {
                            // removes empty strings
                        },
                        _ => match lower_ast_expr(
                            elem,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ) {
                            Ok(expr) => {
                                result.push(expr);
                            },
                            Err(_) => {
                                has_error = true;
                            },
                        },
                    }
                }

                if has_error {
                    return Err(());
                }

                Expr {
                    kind: ExprKind::Format(result),
                    span: e.span,
                }
            },
            ValueKind::Lambda {
                args, value, uid,
                return_ty,
                lowered_from_scoped_let,
            } => {
                let mut hir_args = Vec::with_capacity(args.len());
                let mut arg_names = HashMap::with_capacity(args.len());
                let mut captured_values = vec![];
                let mut has_error = false;

                for ast::ArgDef { name, ty, has_question_mark, attributes } in args.iter() {
                    match arg_names.insert(name.id(), *name) {
                        Some(prev) => {
                            session.push_error(HirError::name_collision(prev, *name));
                            has_error = true;
                        },
                        _ => {},
                    }

                    let ty = if let Some(ty) = ty {
                        if let Ok(mut ty) = lower_ast_ty(
                            ty,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ) {
                            let mut ctxt = ValueCaptureCtxt::new(
                                *uid,
                                &mut captured_values,
                                used_names,
                                name_space,
                            );
                            mut_walker_expr(
                                &mut ty.0,
                                &mut ctxt,
                                &Box::new(find_and_replace_captured_values),
                            );

                            Some(ty)
                        } else {
                            has_error = true;

                            None
                        }
                    } else {
                        None
                    };

                    let attributes = if let Ok(attributes) = lower_ast_attributes(
                        attributes,
                        session,
                        used_names,
                        imports,
                        name_space,
                    ) {
                        attributes
                    } else {
                        has_error = true;

                        vec![]
                    };

                    hir_args.push(Arg {
                        name: *name,
                        ty,
                        has_question_mark: *has_question_mark,
                        attributes,
                    });
                }

                let return_ty = if let Some(ty) = return_ty {
                    if let Ok(ty) = lower_ast_ty(
                        ty,
                        session,
                        used_names,
                        imports,
                        name_space,
                    ) {
                        Some(Box::new(ty))
                    } else {
                        has_error = true;

                        None
                    }
                } else {
                    None
                };

                name_space.push_locals(*uid, arg_names.keys().map(|k| *k).collect());

                try_warn_unnecessary_paren(value, session);

                let value = lower_ast_expr(
                    value,
                    session,
                    used_names,
                    imports,
                    name_space,
                );

                name_space.pop_locals();

                let mut value = value?;
                let mut ctxt = ValueCaptureCtxt::new(
                    *uid,
                    &mut captured_values,
                    used_names,
                    name_space,
                );

                mut_walker_expr(
                    &mut value,
                    &mut ctxt,
                    &Box::new(find_and_replace_captured_values),
                );

                if has_error {
                    return Err(());
                }

                // find unused names
                for (id, id_with_span) in arg_names.iter() {
                    if !used_names.contains(&IdentWithOrigin::new(*id, NameOrigin::Local { origin: *uid })) {
                        session.push_warning(HirWarning::unused_name(*id_with_span, NameBindingType::LambdaArg));
                    }
                }

                Expr {
                    kind: ExprKind::Lambda(Lambda {
                        args: hir_args,
                        value: Box::new(value),
                        captured_values,
                        uid: *uid,
                        return_ty,
                        lowered_from_scoped_let: *lowered_from_scoped_let,
                    }),
                    span: e.span,
                }
            },
            ValueKind::Scope { scope, uid } => {
                let mut name_bindings = Vec::with_capacity(scope.lets.len() + 1);
                let mut has_error = false;

                for let_ in scope.lets.iter() {
                    match &let_.kind {
                        ast::LetKind::Pattern(pattern, expr) => {
                            if let Err(_) = lower_patterns_to_name_bindings(
                                pattern,
                                expr,
                                &mut name_bindings,
                                session,
                            ) {
                                has_error = true;
                            }
                        },
                        ast::LetKind::Incallable {
                            name,
                            generics: _,  // parser guarantees that it's None
                            return_ty,
                            return_val,
                            uid: _,  // ignored
                        } => {
                            name_bindings.push(DestructuredPattern::new(
                                *name,
                                return_val.clone(),
                                return_ty.clone(),
                                /* is_real */ true,
                            ));
                        },

                        // `let add(x: Int, y: Int): Int = x + y;`
                        // -> `let add = \{x: Int, y: Int, x + y};`
                        ast::LetKind::Callable {
                            name,
                            args,
                            generics,
                            return_ty,
                            return_val,
                            uid,
                        } => {
                            name_bindings.push(DestructuredPattern::new(
                                *name,
                                ast::Expr {
                                    kind: ast::ExprKind::Value(ast::ValueKind::Lambda {
                                        args: args.clone(),
                                        value: Box::new(return_val.clone()),
                                        uid: *uid,
                                        return_ty: return_ty.clone().map(|ty| Box::new(ty)),
                                        lowered_from_scoped_let: true,
                                    }),
                                    span: return_val.span,
                                },

                                // `return_ty` of `ast::LetKind::Callable` is that of the function,
                                // not this value itself: `Int` vs `Func(Int, Int, Int)`
                                None,

                                // TODO: is this REAL?
                                /* is_real */ true,
                            ));
                        },
                        _ => todo!(),
                    }
                }

                let mut name_bindings_set = HashSet::with_capacity(name_bindings.len());
                let mut name_collision_checker = HashMap::with_capacity(name_bindings.len());

                for DestructuredPattern { name, .. } in name_bindings.iter() {
                    if let Some(prev) = name_collision_checker.insert(
                        name.id(),
                        *name,
                    ) {
                        session.push_error(HirError::name_collision(prev, *name));
                        has_error = true;
                    }

                    else {
                        name_bindings_set.insert(name.id());
                    }
                }

                name_space.push_locals(*uid, name_bindings_set);

                let mut lets = Vec::with_capacity(name_bindings.len());

                for DestructuredPattern {
                    name, expr, ty, is_real,
                } in name_bindings.iter() {
                    if let Some(s) = ScopedLet::try_new(
                        /* name */ *name,
                        /* value */ lower_ast_expr(
                            expr,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ),
                        /* ty */ ty.as_ref().map(|ty| lower_ast_ty(
                            ty,
                            session,
                            used_names,
                            imports,
                            name_space,
                        )),
                        /* is_real */ *is_real,
                    ) {
                        lets.push(s);
                    }

                    else {
                        has_error = true;
                    }
                }

                let mut original_patterns = vec![];

                // lower patterns
                // these are later used by type checker
                if !has_error {
                    for let_ in scope.lets.iter() {
                        // I don't want errors from this lowerings to bother other lowerings
                        if let ast::LetKind::Pattern(pat, expr) = &let_.kind {
                            if let Ok(pat) = lower_ast_pattern(
                                pat,
                                session,
                                used_names,
                                imports,
                                name_space,
                            ) {
                                original_patterns.push((
                                    pat,
                                    lower_ast_expr(
                                        expr,
                                        session,
                                        used_names,
                                        imports,
                                        name_space,
                                    )?,
                                ));
                            }

                            else {
                                has_error = true;
                            }
                        }
                    }
                }

                let value = lower_ast_expr(
                    &scope.value,
                    session,
                    used_names,
                    imports,
                    name_space,
                );

                for (name, id_with_span) in name_collision_checker.iter() {
                    // if the span is not real, then it's a tmp name generated by compiler
                    if id_with_span.span().is_real && !used_names.contains(
                        &IdentWithOrigin::new(*name, NameOrigin::Local { origin: *uid })
                    ) {
                        session.push_warning(HirWarning::unused_name(*id_with_span, NameBindingType::ScopedLet));
                    }
                }

                name_space.pop_locals();

                if has_error {
                    return Err(());
                }

                Expr {
                    kind: ExprKind::Scope(Scope {
                        lets,
                        original_patterns: original_patterns,
                        uid: *uid,
                        value: Box::new(value?),
                    }),
                    span: e.span,
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
                    imports,
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
                    imports,
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
                imports,
                name_space,
            );
            let rhs = lower_ast_expr(
                rhs,
                session,
                used_names,
                imports,
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
        ast::ExprKind::Field { pre, post } => {
            let pre = lower_ast_expr(
                pre,
                session,
                used_names,
                imports,
                name_space,
            )?;

            Expr {
                kind: ExprKind::Field {
                    pre: Box::new(pre),
                    post: *post,
                },
                span: e.span,
            }
        },
        ast::ExprKind::Call { func, args } => {
            let func = lower_ast_expr(
                func,
                session,
                used_names,
                imports,
                name_space,
            );

            let mut has_error = false;
            let mut hir_args = Vec::with_capacity(args.len());

            for arg in args.iter() {
                try_warn_unnecessary_paren(arg, session);

                if let Ok(arg) = lower_ast_expr(
                    arg,
                    session,
                    used_names,
                    imports,
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
        ast::ExprKind::StructInit { struct_, fields } => {
            let struct_ = lower_ast_expr(
                struct_,
                session,
                used_names,
                imports,
                name_space,
            );
            let mut fields_res = Vec::with_capacity(fields.len());
            let mut has_error = false;

            for ast::StructInitDef { field, value } in fields.iter() {
                try_warn_unnecessary_paren(value, session);

                if let Ok(value) = lower_ast_expr(
                    value,
                    session,
                    used_names,
                    imports,
                    name_space,
                ) {
                    fields_res.push(StructInitField {
                        name: *field,
                        value,
                    });
                }

                else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            Expr {
                kind: ExprKind::StructInit(StructInit {
                    struct_: Box::new(struct_?),
                    fields: fields_res,
                }),
                span: e.span,
            }
        },
        ast::ExprKind::Branch(arms) => {
            let mut branch_arms = Vec::with_capacity(arms.len());
            let mut has_error = false;

            // `if pattern` statements are lowered to `match` statements
            if arms[0].pattern_bind.is_some() {
                // `if pattern PAT = COND { EXPR1 } else { EXPR2 }`
                // -> `match COND { PAT => EXPR1, _ => EXPR2 }`
                return if arms.len() == 2 {
                    let match_expr = ast::Expr {
                        kind: ast::ExprKind::Match {
                            value: Box::new(arms[0].cond.clone().unwrap()),
                            arms: vec![
                                ast::MatchArm {
                                    pattern: arms[0].pattern_bind.clone().unwrap(),
                                    guard: None,
                                    value: arms[0].value.clone(),
                                    uid: Uid::new_match_arm(),
                                },
                                ast::MatchArm {
                                    pattern: ast::Pattern::dummy_wildcard(e.span.into_fake()),
                                    guard: None,
                                    value: arms[1].value.clone(),
                                    uid: Uid::new_match_arm(),
                                },
                            ],
                            is_lowered_from_if_pattern: true,
                        },
                        span: e.span,
                    };

                    lower_ast_expr(
                        &match_expr,
                        session,
                        used_names,
                        imports,
                        name_space,
                    )
                }

                // `if pattern PAT = COND { EXPR } else if ... `
                // -> `match COND { PAT => EXPR, _ => if ... }`
                else {
                    let match_expr = ast::Expr {
                        kind: ast::ExprKind::Match {
                            value: Box::new(arms[0].cond.clone().unwrap()),
                            arms: vec![
                                ast::MatchArm {
                                    pattern: arms[0].pattern_bind.clone().unwrap(),
                                    guard: None,
                                    value: arms[0].value.clone(),
                                    uid: Uid::new_match_arm(),
                                },
                                ast::MatchArm {
                                    pattern: ast::Pattern::dummy_wildcard(e.span.into_fake()),
                                    guard: None,
                                    value: ast::Expr {
                                        kind: ast::ExprKind::Branch(arms[1..].to_vec()),
                                        span: arms[1].span,
                                    },
                                    uid: Uid::new_match_arm(),
                                },
                            ],
                            is_lowered_from_if_pattern: true,
                        },
                        span: e.span,
                    };

                    lower_ast_expr(
                        &match_expr,
                        session,
                        used_names,
                        imports,
                        name_space,
                    )
                };
            }

            for (
                index,
                ast::BranchArm {
                    cond,
                    pattern_bind,
                    value,
                    span,  // of the current `else`
                },
            ) in arms.iter().enumerate() {
                if let Some(cond) = cond {
                    let cond = if let Ok(cond) = lower_ast_expr(
                        cond,
                        session,
                        used_names,
                        imports,
                        name_space,
                    ) {
                        cond
                    } else {
                        has_error = true;
                        continue;
                    };

                    // `if COND1 { EXPR1 } else if pattern PAT = COND2 { EXPR2 } else { EXPR3 }`
                    // -> `if COND1 { EXPR1 } else { if pattern PAT = COND2 { EXPR2 } else { EXPR3 } }`
                    // -> `if COND1 { EXPR1 } else { match COND2 { PAT => EXPR2, _ => EXPR3 } }`
                    if let Some(pattern_bind) = pattern_bind {
                        let else_branch = ast::Expr {
                            kind: ast::ExprKind::Branch(arms[index..].to_vec()),
                            span: *span,
                        };

                        let else_branch = if let Ok(e) = lower_ast_expr(
                            &else_branch,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ) {
                            e
                        } else {
                            has_error = true;
                            break;
                        };

                        branch_arms.push(BranchArm {
                            cond: None,
                            value: else_branch,
                        });

                        break;
                    }

                    else {
                        if let Ok(value) = lower_ast_expr(
                            value,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ) {
                            branch_arms.push(BranchArm {
                                cond: Some(cond),
                                value,
                            });
                        }

                        else {
                            has_error = true;
                        }
                    }
                }

                else {
                    debug_assert!(pattern_bind.is_none());

                    if let Ok(value) = lower_ast_expr(
                        value,
                        session,
                        used_names,
                        imports,
                        name_space,
                    ) {
                        branch_arms.push(BranchArm {
                            cond: None,
                            value,
                        });
                    }

                    else {
                        has_error = true;
                    }
                }
            }

            if has_error {
                return Err(());
            }

            Expr {
                kind: ExprKind::Branch(Branch { arms: branch_arms }),
                span: e.span,
            }
        },
        ast::ExprKind::Match { value, arms, is_lowered_from_if_pattern } => {
            try_warn_unnecessary_paren(value, session);

            let result_value = lower_ast_expr(
                value,
                session,
                used_names,
                imports,
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
                let mut name_bindings_buffer = vec![];
                pattern.get_name_bindings(&mut name_bindings_buffer);

                for def in name_bindings_buffer.iter() {
                    match name_collision_checker.get(&def.id()) {
                        Some(id) => {
                            session.push_error(HirError::name_collision(*def, *id));
                        },
                        None => {
                            name_collision_checker.insert(def.id(), *def);
                        },
                    }

                    name_bindings.insert(def.id());
                }

                name_space.push_locals(
                    *uid,
                    name_bindings,
                );

                let value = lower_ast_expr(
                    value,
                    session,
                    used_names,
                    imports,
                    name_space,
                );

                let pattern = lower_ast_pattern(
                    pattern,
                    session,
                    used_names,
                    imports,
                    name_space,
                );

                let guard = guard.as_ref().map(|g| lower_ast_expr(
                    g,
                    session,
                    used_names,
                    imports,
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
                    is_lowered_from_if_pattern: *is_lowered_from_if_pattern,
                }),
                span: e.span,
            }
        },
        ast::ExprKind::Parenthesis(expr) => {
            try_warn_unnecessary_paren(expr, session);

            lower_ast_expr(
                expr,
                session,
                used_names,
                imports,
                name_space,
            )?
        },
        ast::ExprKind::Error => unreachable!(),
    };

    Ok(res)
}

pub fn try_warn_unnecessary_paren(
    expr: &ast::Expr,
    session: &mut HirSession,
) {
    match &expr.kind {
        ast::ExprKind::Parenthesis(_) => {
            session.push_warning(HirWarning::unnecessary_paren(expr));
        },
        // a scope without any defs
        ast::ExprKind::Value(ast::ValueKind::Scope {
            scope, ..
        }) if scope.has_no_lets() => {
            session.push_warning(HirWarning::unnecessary_paren(expr));
        },
        _ => {},
    }
}

fn fields_from_vec(names: &[IdentWithSpan], span: SpanRange) -> Expr {
    debug_assert!(names.len() > 1);

    if names.len() == 2 {
        Expr {
            kind: ExprKind::Field {
                pre: Box::new(Expr {
                    kind: ExprKind::Identifier(IdentWithOrigin::new(names[0].id(), NameOrigin::Global { origin: None /* dont know yet */ })),
                    span: *names[0].span(),
                }),
                post: FieldKind::Named(names[1]),
            },
            span,
        }
    }

    else {
        Expr {
            kind: ExprKind::Field {
                pre: Box::new(fields_from_vec(&names[..(names.len() - 1)], span)),
                post: FieldKind::Named(names[names.len() - 1]),
            },
            span,
        }
    }
}
