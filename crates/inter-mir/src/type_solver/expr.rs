use crate::{Expr, Session, Type};
use crate::error::{ErrorContext, TypeError};
use sodigy_hir::{AssociatedFunc, FuncPurity};
use sodigy_mir::{Callable, ShortCircuitKind};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_parse::{Field, merge_field_spans};
use sodigy_span::{PolySpanKind, Span};
use sodigy_string::intern_string;
use sodigy_token::Constant;
use std::collections::HashMap;

impl Session {
    // FIXME: there are A LOT OF heap allocations
    //
    // It can solve type of any expression, but the result maybe `Type::Var`.
    // If it finds new type equations while solving, the `Solver` remembers them.
    //
    // If there's no error, it must return the type of the expr: `(Some(ty), false)`.
    // If there're errors, it'll still try to return the type, so that it
    // can find more type errors (only obvious ones).
    pub fn solve_expr(&mut self, expr: &Expr, impure_calls: &mut Vec<Span>) -> (Option<Type>, bool /* has_error */) {
        match expr {
            Expr::Ident(id) => match self.types.get(&id.def_span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    match id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                            // `False` in `Bool.False` has type `Bool`.
                            // TODO: `None` in `Option.None` must have type `Option<T>`, not `Option`.
                            NameKind::EnumVariant { parent } => {
                                return (Some(Type::Data {
                                    constructor_def_span: parent,
                                    constructor_span: Span::None,
                                    args: None,
                                    group_span: None,
                                }), false);
                            },
                            NameKind::PatternNameBind => {
                                self.pattern_name_bindings.insert(id.def_span);
                            },
                            _ => {},
                        },
                        _ => {},
                    }

                    // NOTE: inter-hir must have checked that `id` is a valid expression

                    self.add_type_var(Type::Var { def_span: id.def_span, is_return: false }, Some(id.id));
                    (
                        Some(Type::Var {
                            def_span: id.def_span,
                            is_return: false,
                        }),
                        false,
                    )
                },
            },
            Expr::Constant(Constant::Number { n, .. }) => match n.is_integer {
                true => (
                    Some(Type::Data {
                        constructor_def_span: self.get_lang_item_span("type.Int"),
                        constructor_span: Span::None,
                        args: None,
                        group_span: None,
                    }),
                    false,
                ),
                false => (
                    Some(Type::Data {
                        constructor_def_span: self.get_lang_item_span("type.Number"),
                        constructor_span: Span::None,
                        args: None,
                        group_span: None,
                    }),
                    false,
                ),
            },
            Expr::Constant(Constant::String { binary, .. }) => match *binary {
                true => (
                    Some(Type::Data {
                        constructor_def_span: self.get_lang_item_span("type.List"),
                        constructor_span: Span::None,
                        args: Some(vec![Type::Data {
                            constructor_def_span: self.get_lang_item_span("type.Byte"),
                            constructor_span: Span::None,
                            args: None,
                            group_span: None,
                        }]),
                        group_span: Some(Span::None),
                    }),
                    false,
                ),
                false => (
                    Some(Type::Data {
                        constructor_def_span: self.get_lang_item_span("type.List"),
                        constructor_span: Span::None,
                        args: Some(vec![Type::Data {
                            constructor_def_span: self.get_lang_item_span("type.Char"),
                            constructor_span: Span::None,
                            args: None,
                            group_span: None,
                        }]),
                        group_span: None,
                    }),
                    false,
                ),
            },
            Expr::Constant(Constant::Char { .. }) => (
                Some(Type::Data {
                    constructor_def_span: self.get_lang_item_span("type.Char"),
                    constructor_span: Span::None,
                    args: None,
                    group_span: None,
                }),
                false,
            ),
            Expr::Constant(Constant::Byte { .. }) => (
                Some(Type::Data {
                    constructor_def_span: self.get_lang_item_span("type.Byte"),
                    constructor_span: Span::None,
                    args: None,
                    group_span: None,
                }),
                false,
            ),
            Expr::Constant(Constant::Scalar(_)) => (
                Some(Type::Data {
                    constructor_def_span: self.get_lang_item_span("type.Scalar"),
                    constructor_span: Span::None,
                    args: None,
                    group_span: None,
                }),
                false,
            ),
            Expr::If(r#if) => match r#if.from_short_circuit {
                Some(s) => {
                    let mut has_error = false;
                    let bool_type = Type::Data {
                        constructor_def_span: self.get_lang_item_span("type.Bool"),
                        constructor_span: Span::None,
                        args: None,
                        group_span: None,
                    };
                    let context = match s {
                        ShortCircuitKind::And => ErrorContext::ShortCircuitAndBool,
                        ShortCircuitKind::Or => ErrorContext::ShortCircuitOrBool,
                    };

                    for v in [
                        &r#if.cond,
                        &r#if.true_value,
                        &r#if.false_value,
                    ] {
                        let (v_type, e) = self.solve_expr(v, impure_calls);

                        if e {
                            has_error = true;
                        }

                        if let Some(v_type) = v_type {
                            if let Err(()) = self.solve_supertype(
                                &bool_type,
                                &v_type,
                                false,
                                None,
                                Some(v.error_span_wide()),
                                context.clone(),
                                false,
                            ) {
                                has_error = true;
                            }
                        }
                    }

                    (Some(bool_type), has_error)
                },
                None => {
                    let (cond_type, mut has_error) = self.solve_expr(r#if.cond.as_ref(), impure_calls);

                    if let Some(cond_type) = cond_type {
                        if let Err(()) = self.solve_supertype(
                            &Type::Data {
                                constructor_def_span: self.get_lang_item_span("type.Bool"),
                                constructor_span: Span::None,
                                args: None,
                                group_span: None,
                            },
                            &cond_type,
                            false,
                            None,
                            Some(r#if.cond.error_span_wide()),
                            ErrorContext::IfConditionBool,
                            false,
                        ) {
                            has_error = true;
                        }
                    }

                    match (
                        self.solve_expr(r#if.true_value.as_ref(), impure_calls),
                        self.solve_expr(r#if.false_value.as_ref(), impure_calls),
                    ) {
                        ((Some(true_type), e1), (Some(false_type), e2)) => match self.solve_supertype(
                            &true_type,
                            &false_type,
                            false,
                            Some(r#if.true_value.error_span_wide()),
                            Some(r#if.false_value.error_span_wide()),
                            ErrorContext::IfValueEqual,

                            // If either `true_type <: false_type` or `false_type <: true_type` is satisfied, it's okay.
                            true,
                        ) {
                            Ok(expr_type) => (Some(expr_type), has_error | e1 | e2),
                            Err(()) => (None, true),
                        },
                        _ => (None, true),
                    }
                },
            },
            // 1. value_type == pattern_type
            //    - but we don't check the types of patterns here
            //    - MatchFsm will do that
            // 2. guard_type == bool
            // 3. arm_type == all the other arm_types
            // 4. arm_type == expr_type
            // 5. scrutinee_type == pattern_types
            Expr::Match(r#match) => {
                let (scrutinee_type, mut has_error) = self.solve_expr(r#match.scrutinee.as_ref(), impure_calls);
                let mut arm_types = Vec::with_capacity(r#match.arms.len());

                // TODO: it's okay to fail to infer the types of name bindings
                //       we need some kinda skip list
                for arm in r#match.arms.iter() {
                    if let Some(scrutinee_type) = &scrutinee_type {
                        match self.solve_pattern(&arm.pattern) {
                            (Some(pattern_type), e) => {
                                if let Err(()) = self.solve_supertype(
                                    &scrutinee_type,
                                    &pattern_type,
                                    false,
                                    Some(r#match.scrutinee.error_span_wide()),
                                    Some(arm.pattern.error_span_wide()),
                                    ErrorContext::MatchScrutinee,

                                    // We don't allow `scrutinee_type <: pattern_type`.
                                    // For example, `match todo() { 0 => _ }` is invalid.
                                    false,
                                ) {
                                    has_error = true;
                                }

                                has_error |= e;
                            },
                            (None, _) => {
                                has_error = true;
                            },
                        }
                    }

                    if let Some(guard) = &arm.guard {
                        let (guard_type, e) = self.solve_expr(guard, impure_calls);
                        has_error |= e;

                        if let Some(guard_type) = guard_type {
                            if let Err(()) = self.solve_supertype(
                                &Type::Data {
                                    constructor_def_span: self.get_lang_item_span("type.Bool"),
                                    constructor_span: Span::None,
                                    args: None,
                                    group_span: None,
                                },
                                &guard_type,
                                false,
                                None,
                                Some(guard.error_span_wide()),
                                ErrorContext::MatchGuardBool,
                                false,
                            ) {
                                has_error = true;
                            }
                        }
                    }

                    let (arm_type, e) = self.solve_expr(&arm.value, impure_calls);
                    has_error |= e;

                    if let Some(arm_type) = arm_type {
                        arm_types.push(arm_type);
                    }
                }

                if has_error {
                    (None, true)
                }

                else {
                    // parser guarantees that there's at least 1 arm
                    let mut expr_type = arm_types[0].clone();
                    let mut has_error = false;

                    for i in 1..arm_types.len() {
                        if let Ok(new_expr_type) = self.solve_supertype(
                            &expr_type,
                            &arm_types[i],
                            false,
                            Some(r#match.arms[0].value.error_span_wide()),
                            Some(r#match.arms[i].value.error_span_wide()),
                            ErrorContext::MatchArmEqual,

                            // If either `expr_type <: arg_types[i]` or `arg_types[i] <: expr_type` is satisfied, it's okay.
                            true,
                        ) {
                            expr_type = new_expr_type;
                        }

                        else {
                            has_error = true;
                        }
                    }

                    if has_error {
                        (None, true)
                    }

                    else {
                        (Some(expr_type), false)
                    }
                }
            },
            Expr::Block(block) => {
                let mut has_error = false;

                for r#let in block.lets.iter() {
                    let (_, e) = self.solve_let(r#let, impure_calls);
                    has_error |= e;
                }

                for assert in block.asserts.iter() {
                    if let Err(()) = self.solve_assert(assert, impure_calls) {
                        has_error = true;
                    }
                }

                let (expr_type, e) = self.solve_expr(block.value.as_ref(), impure_calls);
                (expr_type, e || has_error)
            },
            Expr::Field { lhs, fields } => match self.solve_expr(lhs, impure_calls) {
                (Some(lhs_type), has_error) => match self.get_type_of_field(&lhs_type, fields) {
                    Ok(field_type) => (Some(field_type), has_error),
                    Err(()) => (None, true),
                },
                (None, _) => (None, true),
            },
            // 1. Make sure that `lhs` has the fields.
            // 2. Make sure that the field's type and `rhs`' type are the same.
            // 3. Return the type of `lhs`.
            Expr::FieldUpdate { fields, lhs, rhs } => match self.solve_expr(lhs, impure_calls) {
                (Some(lhs_type), mut has_error) => match self.get_type_of_field(&lhs_type, fields) {
                    Ok(field_type) => match self.solve_expr(rhs, impure_calls) {
                        (Some(rhs_type), e) => {
                            has_error |= e;

                            if let Err(()) = self.solve_supertype(
                                &field_type,
                                &rhs_type,
                                false,
                                Some(merge_field_spans(fields)),
                                Some(rhs.error_span_wide()),
                                ErrorContext::FieldUpdate,
                                false,
                            ) {
                                has_error = true;
                            }

                            (Some(lhs_type), has_error)
                        },
                        (None, _) => (Some(lhs_type), true),
                    },
                    Err(()) => (Some(lhs_type), true),
                },
                (None, _) => (None, true),
            },
            Expr::Call { func, args, types: generic_args, given_keyword_args, .. } => {
                let mut has_error = false;
                let mut arg_types = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    match self.solve_expr(arg, impure_calls) {
                        (Some(arg_type), e) => {
                            arg_types.push(arg_type);
                            has_error |= e;
                        },
                        (None, e) => {
                            has_error |= e;
                        },
                    }
                }

                if has_error {
                    return (None, true);
                }

                match func {
                    // The `expr` is `f()` and we know the def_span of `f`.
                    Callable::Static { def_span, span } => match self.types.get(def_span) {
                        // `f` is a function and we have enough information.
                        Some(Type::Func {
                            params,
                            r#return,
                            purity,
                            ..
                        }) => {
                            if let FuncPurity::Impure | FuncPurity::Both = purity {
                                impure_calls.push(*span);
                            }

                            let mut params = params.clone();
                            let mut return_type: Type = *r#return.clone();
                            let generic_params = self.func_shapes.get(def_span).map(
                                |func_shape| func_shape.generics.iter().map(
                                    |generic| generic.name_span
                                ).collect()
                            ).unwrap_or(vec![]);
                            let span = *span;

                            if let Some((generic_args, arg_group_span)) = generic_args {
                                if generic_args.len() != generic_params.len() {
                                    self.type_errors.push(TypeError::WrongNumberOfGenericArgs {
                                        expected: generic_params.len(),
                                        got: generic_args.len(),
                                        param_group_span: self.func_shapes.get(def_span).unwrap().generic_group_span.unwrap_or(Span::None),
                                        arg_group_span: *arg_group_span,
                                    });
                                    return (None, true);
                                }

                                else {
                                    for (generic_param, generic_arg) in generic_params.iter().zip(generic_args.iter()) {
                                        let generic_arg_type_var = Type::GenericArg { call: span, generic: *generic_param };

                                        for param in params.iter_mut() {
                                            param.substitute_generic_param(generic_param, generic_arg);
                                        }

                                        return_type.substitute_generic_param(generic_param, generic_arg);

                                        if let Err(()) = self.solve_supertype(
                                            &generic_arg_type_var,
                                            generic_arg,
                                            false,
                                            None,
                                            Some(generic_arg.error_span_wide()),
                                            ErrorContext::None,
                                            true,
                                        ) {
                                            has_error = true;
                                        }

                                        self.add_type_var(generic_arg_type_var, None);
                                    }
                                }
                            }

                            else if !generic_params.is_empty() {
                                for param in params.iter_mut() {
                                    param.substitute_generic_param_for_arg(span, &generic_params);
                                }

                                return_type.substitute_generic_param_for_arg(span, &generic_params);

                                for generic_param in generic_params.iter() {
                                    self.add_type_var(Type::GenericArg { call: span, generic: *generic_param }, None);
                                }
                            }

                            // It doesn't check arg types if there are wrong number of args.
                            // Whether or not there're type errors with args, it returns the return type.
                            if arg_types.len() != params.len() {
                                has_error = true;
                                self.type_errors.push(TypeError::WrongNumberOfArgs {
                                    expected: params,
                                    got: arg_types,
                                    given_keyword_args: given_keyword_args.to_vec(),
                                    func_span: func.error_span_wide(),
                                    arg_spans: args.iter().map(|arg| arg.error_span_wide()).collect(),
                                });
                            }

                            else {
                                for (i, param) in params.iter().enumerate() {
                                    if let Err(()) = self.solve_supertype(
                                        param,
                                        &arg_types[i],
                                        false,
                                        None,
                                        Some(args[i].error_span_wide()),
                                        ErrorContext::FuncArgs,
                                        false,
                                    ) {
                                        has_error = true;
                                    }
                                }
                            }

                            (Some(return_type), has_error)
                        },
                        // We're sure that `f` is not a function.
                        // For example, `let f = 3; f()`.
                        Some(t @ Type::Data { .. }) => {
                            self.type_errors.push(TypeError::NotCallable {
                                r#type: t.clone(),
                                func_span: *span,
                            });
                            (None, true)
                        },
                        // We only type check/infer monomorphized functions.
                        Some(Type::GenericParam { .. }) => unreachable!(),
                        // This is not a type error because `!` is subtype of every type.
                        Some(t @ Type::Never(_)) => (Some(t.clone()), has_error),
                        // `fn foo(x) = x;`.
                        // When someone calls `foo`, they'll reach this branch.
                        // `def_span` will have `foo`'s span and `v` will have
                        // `x` (the param definition)'s span.
                        Some(v @ (Type::Var { .. } | Type::GenericArg { .. } | Type::Blocked { .. })) => {
                            let v = v.clone();

                            match self.solve_supertype(
                                &Type::Var {
                                    def_span: *def_span,
                                    is_return: true,
                                },
                                &v,
                                false,
                                None,
                                None,
                                ErrorContext::Deep,
                                true,
                            ) {
                                Ok(r#type) => (Some(r#type), has_error),
                                Err(()) => (Some(v), has_error),
                            }
                        },
                        None => todo!(),
                    },
                    Callable::StructInit { def_span, span } => match self.struct_shapes.get(def_span) {
                        Some(s) => {
                            // The compiler checked it when lowering hir to mir.
                            assert_eq!(s.fields.len(), arg_types.len());
                            let s = s.clone();
                            let call_span = *span;
                            let generic_params = s.generics.iter().map(
                                |generic| generic.name_span
                            ).collect::<Vec<_>>();

                            for i in 0..arg_types.len() {
                                let field_type = match self.types.get(&s.fields[i].name_span) {
                                    Some(r#type) => {
                                        let mut r#type = r#type.clone();
                                        r#type.substitute_generic_param_for_arg(call_span, &generic_params);
                                        r#type
                                    },
                                    None => Type::Var { def_span: s.fields[i].name_span, is_return: false },
                                };

                                if let Err(()) = self.solve_supertype(
                                    &field_type,
                                    &arg_types[i],
                                    false,
                                    None,
                                    Some(args[i].error_span_wide()),
                                    ErrorContext::StructFields,
                                    false,
                                ) {
                                    has_error = true;
                                }
                            }

                            if !generic_params.is_empty() {
                                for generic_param in generic_params.iter() {
                                    self.add_type_var(Type::GenericArg { call: call_span, generic: *generic_param }, None);
                                }
                            }

                            let (args, group_span) = if s.generics.is_empty() {
                                (None, None)
                            } else {
                                (Some(s.generics.iter().map(
                                    |generic| {
                                        match self.generic_args.get(&(call_span, generic.name_span)) {
                                            Some(r#type) => r#type.clone(),
                                            None => Type::GenericArg { call: call_span, generic: generic.name_span },
                                        }
                                    }
                                ).collect()), Some(Span::None))
                            };

                            (
                                Some(Type::Data {
                                    constructor_def_span: *def_span,
                                    constructor_span: Span::None,
                                    args,
                                    group_span,
                                }),
                                has_error,
                            )
                        },

                        // This is kinda Internal Compiler Error.
                        // inter-hir must check whether a struct constructor is from `NameKind::Struct`.
                        None => unreachable!(),
                    },
                    Callable::TupleInit { .. } => (
                        Some(Type::Data {
                            constructor_def_span: self.get_lang_item_span("type.Tuple"),
                            constructor_span: Span::None,
                            args: Some(arg_types),

                            // this is for the type annotation, hence None
                            group_span: Some(Span::None),
                        }),
                        has_error,
                    ),
                    Callable::ListInit { group_span } => {
                        if arg_types.is_empty() {
                            let type_var = Type::GenericArg { call: *group_span, generic: self.get_lang_item_span("built_in.init_list.generic.0") };
                            self.add_type_var(type_var.clone(), None);

                            let r#type = Type::Data {
                                constructor_def_span: self.get_lang_item_span("type.List"),
                                constructor_span: Span::None,
                                args: Some(vec![type_var]),

                                // this is for the type annotation, hence None
                                group_span: Some(Span::None),
                            };
                            (Some(r#type), false)
                        }

                        else {
                            let mut elem_type = arg_types[0].clone();
                            let mut has_error = false;

                            for i in 1..arg_types.len() {
                                if let Ok(new_elem_type) = self.solve_supertype(
                                    &elem_type,
                                    &arg_types[i],
                                    false,
                                    Some(args[0].error_span_wide()),
                                    Some(args[i].error_span_wide()),
                                    ErrorContext::ListElementEqual,

                                    // If either `elem_type <: arg_types[i]` or `arg_types[i] <: elem_type` is satisfied, it's okay.
                                    true,
                                ) {
                                    elem_type = new_elem_type;
                                }

                                else {
                                    has_error = true;
                                }
                            }

                            let r#type = Type::Data {
                                constructor_def_span: self.get_lang_item_span("type.List"),
                                constructor_span: Span::None,
                                args: Some(vec![elem_type]),

                                // this is for the type annotation, hence None
                                group_span: Some(Span::None),
                            };
                            (Some(r#type), has_error)
                        }
                    },
                    Callable::Dynamic(func) => {
                        let (func_type, mut has_error) = match self.solve_expr(func, impure_calls) {
                            (Some(func_type), has_error) => (func_type, has_error),
                            (None, has_error) => {
                                return (None, has_error);
                            },
                        };

                        match func_type {
                            // TODO: What if there's a callable `Type::Data`?
                            Type::Data { .. } => {
                                self.type_errors.push(TypeError::NotCallable {
                                    r#type: func_type.clone(),
                                    func_span: func.error_span_wide(),
                                });
                                return (None, true);
                            },

                            // We'll only type check/infer monomorphized functions.
                            Type::GenericParam { .. } => unreachable!(),

                            Type::Func { params, r#return, purity, .. } => {
                                if let FuncPurity::Impure | FuncPurity::Both = purity {
                                    impure_calls.push(func.error_span_wide());
                                }

                                // It doesn't check arg types if there are wrong number of args.
                                // Whether or not there're type errors with args, it returns the return type.
                                if arg_types.len() != params.len() {
                                    has_error = true;
                                    self.type_errors.push(TypeError::WrongNumberOfArgs {
                                        expected: params,
                                        got: arg_types,
                                        given_keyword_args: given_keyword_args.to_vec(),
                                        func_span: func.error_span_wide(),
                                        arg_spans: args.iter().map(|arg| arg.error_span_wide()).collect(),
                                    });
                                }

                                else {
                                    for i in 0..params.len() {
                                        if let Err(()) = self.solve_supertype(
                                            &params[i],
                                            &arg_types[i],
                                            false,
                                            None,
                                            Some(args[i].error_span_wide()),
                                            ErrorContext::FuncArgs,
                                            false,
                                        ) {
                                            has_error = true;
                                        }
                                    }
                                }

                                (Some(*r#return.clone()), has_error)
                            },

                            // This is difficult...
                            // `let x = { ... }; let y = x();`
                            // Let's say we don't know the type of `x` and we want to infer the type of `y`.
                            // When we look at `x()`, we'll reach this branch (with `func_type = Type::Var(x)`).
                            // We can't create a type equation here because there's no direct relationship between
                            // TypeVar(x) and TypeVar(y). TypeVar(x)'s return type is equal to TypeVar(y), but there's
                            // no way to represent "TypeVar(x)'s return type".
                            Type::Var { def_span: span, .. } | Type::GenericArg { call: span, .. } => {
                                self.blocked_type_vars.insert(span);
                                (Some(Type::Blocked { origin: span }), has_error)
                            },

                            t @ Type::Blocked { .. } => (Some(t), has_error),
                            _ => panic!("TODO: {func:?}, {func_type:?}"),
                        }
                    },
                }
            },
        }
    }

    pub fn get_type_of_field(&mut self, r#type: &Type, field: &[Field]) -> Result<Type, ()> {
        let mut field_type = None;

        // Let's say there's a struct `Game<T, U>` and `r#type` is `Game<Int, String>`.
        // If the `field_type` is `T`, we have to replace `T` with `Int`.
        // This map remembers the connection between generic params and generic args.
        // It looks like `{ T: Int, U: String }`.
        let mut generic_map: HashMap<Span, &Type> = HashMap::new();

        match r#type {
            Type::Data { constructor_def_span: def_span, args, .. } => {
                if *def_span == self.get_lang_item_span("type.Tuple") {
                    let args = args.as_ref().unwrap();

                    match &field[0] {
                        Field::Name { name, .. } => {
                            for i in 0..args.len() {
                                let i_s = format!("_{i}");

                                if name.eq(i_s.as_bytes()) {
                                    field_type = Some(args[i].clone());
                                    break;
                                }
                            }
                        },
                        Field::Index(i) => {
                            let i = if *i < 0 { (*i + args.len() as i64) as usize } else { *i as usize };
                            field_type = Some(args[i].clone());
                        },
                        _ => todo!(),
                    }
                }

                else if let Some(struct_shape) = self.struct_shapes.get(def_span) {
                    if let Some(args) = args {
                        for (generic_param, generic_arg) in struct_shape.generics.iter().zip(args.iter()) {
                            generic_map.insert(generic_param.name_span, generic_arg);
                        }
                    }

                    match &field[0] {
                        Field::Name { name, name_span, .. } => {
                            for field in struct_shape.fields.iter() {
                                if field.name == *name {
                                    match self.types.get(&field.name_span) {
                                        Some(r#type) => {
                                            field_type = Some(r#type.clone());
                                        },
                                        None => {
                                            field_type = Some(Type::Var { def_span: field.name_span, is_return: false });
                                        },
                                    }

                                    break;
                                }
                            }
                        },
                        Field::Index(i) => {
                            let i = if *i < 0 { (*i + struct_shape.fields.len() as i64) as usize } else { *i as usize };
                            let name_span = struct_shape.fields[i].name_span;

                            match self.types.get(&name_span) {
                                Some(r#type) => {
                                    field_type = Some(r#type.clone());
                                },
                                None => {
                                    field_type = Some(Type::Var { def_span: name_span, is_return: false });
                                },
                            }
                        },
                        _ => todo!(),
                    }
                }

                if let Field::Name { name, name_span, .. } = &field[0] && field_type.is_none() {
                    if let Some(item_shape) = self.get_item_shape(*def_span) {
                        // `x.unwrap()` is desugared to `associated_func::unwrap::pure::1(x)`.
                        // `associated_func::unwrap::pure::1` is a poly-generic function and we can
                        // easily reference the function with its name.
                        if let Some(AssociatedFunc { params, is_pure, .. }) = item_shape.associated_funcs().get(name) {
                            let func_name = name.unintern_or_default(&self.intermediate_dir);
                            let purity = if *is_pure { "pure" } else { "impure" };
                            let poly_name = intern_string(
                                format!("associated_func::{func_name}::{purity}::{params}").as_bytes(),
                                &self.intermediate_dir,
                            ).unwrap();

                            field_type = Some(Type::Func {
                                fn_span: Span::None,
                                group_span: Span::None,
                                // Type of `x.unwrap` is `Fn() -> T`, not `Fn(Option<T>) -> T`
                                params: (1..*params).map(
                                    |i| Type::GenericArg {
                                        call: *name_span,
                                        generic: Span::Poly {
                                            name: poly_name,
                                            kind: PolySpanKind::Param(i),
                                            monomorphize_id: None,
                                        },
                                    }
                                ).collect(),
                                r#return: Box::new(Type::GenericArg {
                                    call: *name_span,
                                    generic: Span::Poly {
                                        name: poly_name,
                                        kind: PolySpanKind::Return,
                                        monomorphize_id: None,
                                    },
                                }),
                                purity: if *is_pure {
                                    FuncPurity::Pure
                                } else {
                                    FuncPurity::Impure
                                },
                            });
                        }
                    }
                }
            },
            // `Type::Blocked` exists exactly for this reason.
            // Read the documentation at `crates/mir/src/type.rs`.
            Type::Var { def_span: span, .. } |
            Type::GenericArg { call: span, .. } |
            Type::Blocked { origin: span } => {
                self.blocked_type_vars.insert(*span);
                return Ok(Type::Blocked { origin: *span });
            },
            _ => panic!("TODO: {type:?}"),
        }

        match field_type {
            Some(mut field_type) => {
                for (generic_param, generic_arg) in generic_map.iter() {
                    field_type.substitute_generic_param(generic_param, generic_arg);
                }

                if field.len() == 1 {
                    Ok(field_type)
                }

                else {
                    self.get_type_of_field(&field_type, &field[1..])
                }
            },
            None => {
                // an error..??
                panic!("{type:?}\n{field:?}")
            },
        }
    }
}
