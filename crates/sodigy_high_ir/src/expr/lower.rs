use super::{Branch, BranchArm, Expr, ExprKind, Lambda, LocalDef, Match, MatchArm, Scope, StructInit, StructInitField};
use crate::lower_ast_ty;
use crate::err::HirError;
use crate::func::Arg;
use crate::names::{IdentWithOrigin, NameBindingType, NameOrigin, NameSpace};
use crate::pattern::{lower_patterns_to_name_bindings, lower_ast_pattern};
use crate::session::HirSession;
use crate::warn::HirWarning;
use sodigy_ast::{self as ast, IdentWithSpan, ValueKind};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_test::sodigy_assert;
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
                                *names[0].id(), NameOrigin::Global { origin: None },
                            )),
                            span: *span,
                        }
                    }

                    else {
                        Expr {
                            kind: ExprKind::Path {
                                head: Box::new(Expr {
                                    kind: ExprKind::Identifier(IdentWithOrigin::new(
                                        *names[0].id(), NameOrigin::Global { origin: None },
                                    )),
                                    span: *names[0].span(),
                                }),
                                tail: names[1..].to_vec(),
                            },

                            // it points to the `import` statement
                            span: *span,
                        }
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
                            s,
                            is_binary: false,
                        }) if s.is_empty() => {
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
            } => {
                let mut hir_args = Vec::with_capacity(args.len());
                let mut arg_names = HashMap::with_capacity(args.len());
                let mut captured_names = vec![];
                let mut has_error = false;

                for ast::ArgDef { name, ty, has_question_mark } in args.iter() {
                    match arg_names.insert(*name.id(), *name) {
                        Some(prev) => {
                            session.push_error(HirError::name_collision(prev, *name));
                            has_error = true;
                        },
                        _ => {},
                    }

                    // TODO: what if `ty` has captured names?
                    // e.g: `def foo<T>() = \{x: T, x};`
                    // e.g: `def foo(t: Type) = \{x:t, x};`
                    let ty = if let Some(ty) = ty {
                        if let Ok(mut ty) = lower_ast_ty(
                            ty,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ) {
                            find_and_replace_captured_names(
                                &mut ty.0,
                                *uid,
                                &mut captured_names,
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

                find_and_replace_captured_names(
                    &mut value,
                    *uid,
                    &mut captured_names,
                    used_names,
                    name_space,
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
                        captured_names,
                        uid: *uid,
                    }),
                    span: e.span,
                }
            },
            ValueKind::Scope { scope, uid } => {
                let mut has_error = false;
                let mut name_bindings = vec![];

                // step 1. simple check on `ast::Pattern`s.
                // convert ast::Pattern to name bindings.
                // also collect names
                for ast::LocalDef {
                    pattern, value, ..
                } in scope.defs.iter() {
                    if let Err(_) = lower_patterns_to_name_bindings(
                        pattern,
                        value,
                        &mut name_bindings,
                        session,
                    ) {
                        has_error = true;
                    }
                }

                name_space.push_locals(*uid, name_bindings.iter().map(
                    |(id, _, _)| *id.id()
                ).collect());

                let mut original_patterns = vec![];

                for ast::LocalDef {
                    pattern, value, ..
                } in scope.defs.iter() {
                    match (
                        lower_ast_pattern(
                            pattern,
                            session,
                        ),
                        lower_ast_expr(
                            value,
                            session,
                            used_names,
                            imports,
                            name_space,
                        ),
                    ) {
                        (Ok(pat), Ok(value)) => {
                            original_patterns.push((pat, value));
                        },
                        _ => {
                            has_error = true;
                        },
                    }
                }

                let mut local_defs = Vec::with_capacity(name_bindings.len());

                // TODO: some `expr`s are lowered twice
                for (name, expr, is_real) in name_bindings.iter() {
                    if let Ok(value) = lower_ast_expr(
                        expr,
                        session,
                        used_names,
                        imports,
                        name_space,
                    ) {
                        local_defs.push(LocalDef {
                            name: *name,
                            value,
                            is_real: *is_real,
                        });
                    }

                    else {
                        has_error = true;
                    }
                }

                let value = lower_ast_expr(
                    &scope.value,
                    session,
                    used_names,
                    imports,
                    name_space,
                );

                for (name, _, is_real) in name_bindings.iter() {
                    if !*is_real {
                        continue;
                    }

                    if !used_names.contains(&IdentWithOrigin::new(*name.id(), NameOrigin::Local { origin: *uid })) {
                        session.push_warning(HirWarning::unused_name(
                            *name,
                            NameBindingType::LocalScope,
                        ));
                    }
                }

                name_space.pop_locals();

                if has_error {
                    return Err(());
                }

                Expr {
                    kind: ExprKind::Scope(Scope {
                        original_patterns,
                        defs: local_defs,
                        value: Box::new(value?),
                        uid: *uid,
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
        // it prettifies ast::Path
        // `a.b.c` -> ast: `Path { pre: Path { pre: a, post: b }, post: c }`
        // `a.b.c` -> hir: `Path { head: a, tail: [b, c] }`
        ast::ExprKind::Path { pre, post } => {
            let head = lower_ast_expr(
                pre,
                session,
                used_names,
                imports,
                name_space,
            )?;

            if let Expr {
                kind: ExprKind::Path { head: i_head, tail: mut i_tail },
                span: i_span,
            } = head {
                i_tail.push(*post);

                Expr {
                    kind: ExprKind::Path {
                        head: i_head,
                        tail: i_tail,
                    },
                    span: i_span,
                }
            }

            else {
                Expr {
                    kind: ExprKind::Path {
                        head: Box::new(head),
                        tail: vec![*post],
                    },
                    span: e.span,
                }
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

            for ast::BranchArm {
                cond,
                let_bind,
                value,
            } in arms.iter() {
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

                    if let Some(let_bind) = let_bind {
                        session.push_error(HirError::todo("if-let", e.span));
                        has_error = true;
                        continue;
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
                                let_bind: None,
                                value,
                            });
                        }

                        else {
                            has_error = true;
                        }
                    }
                }

                else {
                    sodigy_assert!(let_bind.is_none());

                    if let Ok(value) = lower_ast_expr(
                        value,
                        session,
                        used_names,
                        imports,
                        name_space,
                    ) {
                        branch_arms.push(BranchArm {
                            cond: None,
                            let_bind: None,
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
        ast::ExprKind::Match { value, arms } => {
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
                    imports,
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
    };

    Ok(res)
}

fn find_and_replace_captured_names(
    ex: &mut Expr,
    lambda_uid: Uid,
    captured_names: &mut Vec<IdentWithOrigin>,
    used_names: &mut HashSet<IdentWithOrigin>,
    name_space: &mut NameSpace,
) {
    match &mut ex.kind {
        ExprKind::Integer(_)
        | ExprKind::Ratio(_)
        | ExprKind::String { .. }
        | ExprKind::Char(_) => { return; },
        ExprKind::Identifier(id_ori) => {
            let origin = *id_ori.origin();

            // checks whether this id should be captured or not
            match origin {
                NameOrigin::Prelude   // not captured 
                | NameOrigin::Global { .. }  // not captured
                | NameOrigin::Captured { .. }  // captured, but it'll handle names in Lambda.captured_names
                => {
                    return;
                },
                NameOrigin::FuncArg { .. }
                | NameOrigin::FuncGeneric { .. } => {
                    /* must be captured */
                },
                NameOrigin::Local { origin: local_origin } => {
                    // has to see whether it's captured or not
                    // there are 2 cases: Lambda in a Scope, Scope in a Lambda
                    // first case: that's a captured name and the scope is still in the name_space
                    // second case: that's not a captured name and we can ignore that
                    if !name_space.has_this_local_uid(local_origin) {
                        return;
                    }
                },
            }

            let id = *id_ori.id();
            let mut name_index = None;

            // linear search is fine because `captured_names` is small enough in most cases
            for (ind, id_ori_) in captured_names.iter().enumerate() {
                let id_ = *id_ori_.id();
                let origin_ = *id_ori_.origin();

                if (id, origin) == (id_, origin_) {
                    name_index = Some(ind);
                    break;
                }
            }

            if name_index == None {
                name_index = Some(captured_names.len());
                captured_names.push(IdentWithOrigin::new(id, origin));
            }

            let name_index = name_index.unwrap();
            id_ori.set_origin(NameOrigin::Captured {
                lambda: lambda_uid,
                index: name_index,
            });
            used_names.insert(id_ori.clone());
        },
        ExprKind::Call { func, args } => {
            find_and_replace_captured_names(
                func,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );

            for arg in args.iter_mut() {
                find_and_replace_captured_names(
                    arg,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::List(elems)
        | ExprKind::Tuple(elems)
        | ExprKind::Format(elems) => {
            for elem in elems.iter_mut() {
                find_and_replace_captured_names(
                    elem,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::Scope(Scope { defs, value, .. }) => {
            find_and_replace_captured_names(
                value,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );

            // TODO: do we have to look into patterns?
            for LocalDef { value, .. } in defs.iter_mut() {
                find_and_replace_captured_names(
                    value,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );
            }
        },

        // lambda A in lambda B
        // let's say A captures name x
        // case 1: B also captures x
        //    -> has to modify `captured_names` of A
        // case 2: x is defined in B
        //    -> don't have to do anything
        ExprKind::Lambda(Lambda {
            args,
            value,
            captured_names: lambda_captured_names,
            ..
        }) => {
            for arg in args.iter_mut() {
                if let Some(ty) = &mut arg.ty {
                    find_and_replace_captured_names(
                        &mut ty.0,
                        lambda_uid,
                        captured_names,
                        used_names,
                        name_space,
                    );
                }
            }

            find_and_replace_captured_names(
                value,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );

            for captured_name in lambda_captured_names.iter_mut() {
                let mut dummy_expr = Expr {
                    kind: ExprKind::Identifier(*captured_name),
                    span: SpanRange::dummy(),
                };

                find_and_replace_captured_names(
                    &mut dummy_expr,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );

                if let ExprKind::Identifier(captured_name_modified) = dummy_expr.kind {
                    *captured_name = captured_name_modified;
                }
            }
        },
        ExprKind::Match(Match {
            arms,
            value,
        }) => {
            find_and_replace_captured_names(
                value,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );

            // TODO: handle patterns and guards
            for MatchArm { value, .. } in arms.iter_mut() {
                find_and_replace_captured_names(
                    value,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::Branch(Branch { arms }) => {
            for BranchArm {
                cond, let_bind, value,
            } in arms.iter_mut() {
                if let Some(cond) = cond {
                    find_and_replace_captured_names(
                        cond,
                        lambda_uid,
                        captured_names,
                        used_names,
                        name_space,
                    );
                }

                if let Some(let_bind) = let_bind {
                    find_and_replace_captured_names(
                        let_bind,
                        lambda_uid,
                        captured_names,
                        used_names,
                        name_space,
                    );
                }

                find_and_replace_captured_names(
                    value,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::StructInit(StructInit {
            struct_,
            fields
        }) => {
            find_and_replace_captured_names(
                struct_,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );

            for StructInitField { value, .. } in fields.iter_mut() {
                find_and_replace_captured_names(
                    value,
                    lambda_uid,
                    captured_names,
                    used_names,
                    name_space,
                );
            }
        },
        ExprKind::Path {
            head, ..
        } => {
            find_and_replace_captured_names(
                head,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );
        },
        ExprKind::PrefixOp(_, value)
        | ExprKind::PostfixOp(_, value) => {
            find_and_replace_captured_names(
                value,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );
        },
        ExprKind::InfixOp(_, lhs, rhs) => {
            find_and_replace_captured_names(
                lhs,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );
            find_and_replace_captured_names(
                rhs,
                lambda_uid,
                captured_names,
                used_names,
                name_space,
            );
        },
    }
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
        }) if scope.has_no_defs() => {
            session.push_warning(HirWarning::unnecessary_paren(expr));
        },
        _ => {},
    }
}
