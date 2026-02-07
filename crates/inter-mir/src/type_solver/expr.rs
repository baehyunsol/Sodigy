use super::TypeSolver;
use crate::{Expr, Type};
use crate::error::{ErrorContext, TypeError};
use sodigy_hir::FuncPurity;
use sodigy_mir::{Callable, ShortCircuitKind};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_parse::{Field, merge_field_spans};
use sodigy_span::{PolySpanKind, Span};
use sodigy_string::intern_string;
use sodigy_token::Constant;
use std::collections::HashMap;

impl TypeSolver {
    // FIXME: there are A LOT OF heap allocations
    //
    // It can solve type of any expression, but the result maybe `Type::Var`.
    // If it finds new type equations while solving, the `Solver` remembers them.
    //
    // If there's no error, it must return the type of the expr: `(Some(ty), false)`.
    // If there're errors, it'll still try to return the type, so that it
    // can find more type errors (only obvious ones).
    pub fn solve_expr(
        &mut self,
        expr: &Expr,
        impure_calls: &mut Vec<Span>,
        types: &mut HashMap<Span, Type>,
        generic_args: &mut HashMap<(Span, Span), Type>,
    ) -> (Option<Type>, bool /* has_error */) {
        match expr {
            Expr::Ident(id) => match types.get(&id.def_span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    match id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                            // `False` in `Bool.False` has type `Bool`.
                            // TODO: `None` in `Option.None` must have type `Option<T>`, not `Option`.
                            NameKind::EnumVariant { parent } => {
                                return (Some(Type::Static { def_span: parent, span: Span::None }), false);
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
                    Some(Type::Static {
                        def_span: self.get_lang_item_span("type.Int"),
                        span: Span::None,
                    }),
                    false,
                ),
                false => (
                    Some(Type::Static {
                        def_span: self.get_lang_item_span("type.Number"),
                        span: Span::None,
                    }),
                    false,
                ),
            },
            Expr::Constant(Constant::String { binary, .. }) => match *binary {
                true => (
                    Some(Type::Param {
                        constructor_def_span: self.get_lang_item_span("type.List"),
                        constructor_span: Span::None,
                        args: vec![Type::Static {
                            def_span: self.get_lang_item_span("type.Byte"),
                            span: Span::None,
                        }],
                        group_span: Span::None,
                    }),
                    false,
                ),
                false => (
                    Some(Type::Param {
                        constructor_def_span: self.get_lang_item_span("type.List"),
                        constructor_span: Span::None,
                        args: vec![Type::Static {
                            def_span: self.get_lang_item_span("type.Char"),
                            span: Span::None,
                        }],
                        group_span: Span::None,
                    }),
                    false,
                ),
            },
            Expr::Constant(Constant::Char { .. }) => (
                Some(Type::Static {
                    def_span: self.get_lang_item_span("type.Char"),
                    span: Span::None,
                }),
                false,
            ),
            Expr::Constant(Constant::Byte { .. }) => (
                Some(Type::Static {
                    def_span: self.get_lang_item_span("type.Byte"),
                    span: Span::None,
                }),
                false,
            ),
            Expr::If(r#if) => match r#if.from_short_circuit {
                Some(s) => {
                    let mut has_error = false;
                    let bool_type = Type::Static {
                        def_span: self.get_lang_item_span("type.Bool"),
                        span: Span::None,
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
                        let (v_type, e) = self.solve_expr(v, impure_calls, types, generic_args);

                        if e {
                            has_error = true;
                        }

                        if let Some(v_type) = v_type {
                            if let Err(()) = self.solve_supertype(
                                &bool_type,
                                &v_type,
                                types,
                                generic_args,
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
                    let (cond_type, mut has_error) = self.solve_expr(r#if.cond.as_ref(), impure_calls, types, generic_args);

                    if let Some(cond_type) = cond_type {
                        if let Err(()) = self.solve_supertype(
                            &Type::Static {
                                def_span: self.get_lang_item_span("type.Bool"),
                                span: Span::None,
                            },
                            &cond_type,
                            types,
                            generic_args,
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
                        self.solve_expr(r#if.true_value.as_ref(), impure_calls, types, generic_args),
                        self.solve_expr(r#if.false_value.as_ref(), impure_calls, types, generic_args),
                    ) {
                        ((Some(true_type), e1), (Some(false_type), e2)) => match self.solve_supertype(
                            &true_type,
                            &false_type,
                            types,
                            generic_args,
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
                let (scrutinee_type, mut has_error) = self.solve_expr(r#match.scrutinee.as_ref(), impure_calls, types, generic_args);
                let mut arm_types = Vec::with_capacity(r#match.arms.len());

                // TODO: it's okay to fail to infer the types of name bindings
                //       we need some kinda skip list
                for arm in r#match.arms.iter() {
                    if let Some(scrutinee_type) = &scrutinee_type {
                        match self.solve_pattern(&arm.pattern, types, generic_args) {
                            (Some(pattern_type), e) => {
                                if let Err(()) = self.solve_supertype(
                                    &scrutinee_type,
                                    &pattern_type,
                                    types,
                                    generic_args,
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
                        let (guard_type, e) = self.solve_expr(guard, impure_calls, types, generic_args);
                        has_error |= e;

                        if let Some(guard_type) = guard_type {
                            if let Err(()) = self.solve_supertype(
                                &Type::Static {
                                    def_span: self.get_lang_item_span("type.Bool"),
                                    span: Span::None,
                                },
                                &guard_type,
                                types,
                                generic_args,
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

                    let (arm_type, e) = self.solve_expr(&arm.value, impure_calls, types, generic_args);
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
                            types,
                            generic_args,
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
                    let (_, e) = self.solve_let(r#let, impure_calls, types, generic_args);
                    has_error |= e;
                }

                for assert in block.asserts.iter() {
                    if let Err(()) = self.solve_assert(assert, impure_calls, types, generic_args) {
                        has_error = true;
                    }
                }

                let (expr_type, e) = self.solve_expr(block.value.as_ref(), impure_calls, types, generic_args);
                (expr_type, e || has_error)
            },
            Expr::Field { lhs, fields } => match self.solve_expr(lhs, impure_calls, types, generic_args) {
                (Some(lhs_type), has_error) => match self.get_type_of_field(&lhs_type, fields, types, generic_args) {
                    Ok(field_type) => (Some(field_type), has_error),
                    Err(()) => (None, true),
                },
                (None, _) => (None, true),
            },
            // 1. Make sure that `lhs` has the fields.
            // 2. Make sure that the field's type and `rhs`' type are the same.
            // 3. Return the type of `lhs`.
            Expr::FieldUpdate { fields, lhs, rhs } => match self.solve_expr(lhs, impure_calls, types, generic_args) {
                (Some(lhs_type), mut has_error) => match self.get_type_of_field(&lhs_type, fields, types, generic_args) {
                    Ok(field_type) => match self.solve_expr(rhs, impure_calls, types, generic_args) {
                        (Some(rhs_type), e) => {
                            has_error |= e;

                            if let Err(()) = self.solve_supertype(
                                &field_type,
                                &rhs_type,
                                types,
                                generic_args,
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
            // 1. we can solve types of args
            // 2. if callable is...
            //    - a function without generic
            //      - every arg must have a concrete type, so does the return type
            //      - it calls `equal` for all args, and returns the return type
            //    - a generic function
            //      - it first converts `GenericParam` to `GenericArg` and does what
            //        a non-generic function does
            Expr::Call { func, args, generic_defs, given_keyword_arguments, .. } => {
                let mut has_error = false;
                let mut arg_types = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    match self.solve_expr(arg, impure_calls, types, generic_args) {
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
                    Callable::Static { def_span, span } => match types.get(def_span) {
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
                            let span = *span;

                            if !generic_defs.is_empty() {
                                for param in params.iter_mut() {
                                    param.substitute_generic_def(span, &generic_defs);
                                }

                                return_type.substitute_generic_def(span, &generic_defs);

                                for generic_def in generic_defs.iter() {
                                    self.add_type_var(Type::GenericArg { call: span, generic: *generic_def }, None);
                                }
                            }

                            // It doesn't check arg types if there are wrong number of args.
                            // Whether or not there're type errors with args, it returns the return type.
                            if arg_types.len() != params.len() {
                                has_error = true;
                                self.errors.push(TypeError::WrongNumberOfArguments {
                                    expected: params,
                                    got: arg_types,
                                    given_keyword_arguments: given_keyword_arguments.to_vec(),
                                    func_span: func.error_span_wide(),
                                    arg_spans: args.iter().map(|arg| arg.error_span_wide()).collect(),
                                });
                            }

                            else {
                                for (i, param) in params.iter().enumerate() {
                                    if let Err(()) = self.solve_supertype(
                                        param,
                                        &arg_types[i],
                                        types,
                                        generic_args,
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
                        Some(t @ (Type::Static { .. } | Type::Tuple { .. } | Type::Param { .. })) => {
                            self.errors.push(TypeError::NotCallable {
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
                                types,
                                generic_args,
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

                            for i in 0..arg_types.len() {
                                let field_type = match types.get(&s.fields[i].name_span) {
                                    Some(r#type) => r#type.clone(),
                                    None => Type::Var { def_span: s.fields[i].name_span, is_return: false },
                                };

                                if let Err(()) = self.solve_supertype(
                                    &field_type,
                                    &arg_types[i],
                                    types,
                                    generic_args,
                                    false,
                                    None,
                                    Some(args[i].error_span_wide()),
                                    ErrorContext::StructFields,
                                    false,
                                ) {
                                    has_error = true;
                                }
                            }

                            // TODO: If it's generic, the type has to be `Type::Param`.
                            //       But there's no way we can check whether it's generic or not
                            (Some(Type::Static { def_span: *def_span, span: Span::None }), has_error)
                        },

                        // This is kinda Internal Compiler Error.
                        // inter-hir must check whether a struct constructor is from `NameKind::Struct`.
                        None => unreachable!(),
                    },
                    Callable::TupleInit { .. } => (
                        Some(Type::Tuple {
                            args: arg_types,

                            // this is for the type annotation, hence None
                            group_span: Span::None,
                        }),
                        has_error,
                    ),
                    Callable::ListInit { group_span } => {
                        // We can treat a list initialization (`[1, 2, 3]`) like calling a
                        // function with variadic arguments (`list.init(1, 2, 3)`).
                        // Here, `list.init` is a generic function `fn init<T>(args) -> [T]`.
                        // Then, an empty initialization is like calling a generic function
                        // but we don't know its generic yet.
                        if arg_types.is_empty() {
                            let type_var = Type::GenericArg { call: *group_span, generic: self.get_lang_item_span("built_in.init_list.generic.0") };
                            self.add_type_var(type_var.clone(), None);

                            let r#type = Type::Param {
                                constructor_def_span: self.get_lang_item_span("type.List"),
                                constructor_span: Span::None,
                                args: vec![type_var],

                                // this is for the type annotation, hence None
                                group_span: Span::None,
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
                                    types,
                                    generic_args,
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

                            let r#type = Type::Param {
                                constructor_def_span: self.get_lang_item_span("type.List"),
                                constructor_span: Span::None,
                                args: vec![elem_type],

                                // this is for the type annotation, hence None
                                group_span: Span::None,
                            };
                            (Some(r#type), has_error)
                        }
                    },
                    Callable::Dynamic(func) => {
                        let (func_type, mut has_error) = match self.solve_expr(func, impure_calls, types, generic_args) {
                            (Some(func_type), has_error) => (func_type, has_error),
                            (None, has_error) => {
                                return (None, has_error);
                            },
                        };

                        match func_type {
                            // TODO: What if there's a callable `Type::Static()` or `Type::Param {}`?
                            Type::Static { .. } | Type::Tuple { .. } | Type::Param { .. } => {
                                self.errors.push(TypeError::NotCallable {
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
                                    self.errors.push(TypeError::WrongNumberOfArguments {
                                        expected: params,
                                        got: arg_types,
                                        given_keyword_arguments: given_keyword_arguments.to_vec(),
                                        func_span: func.error_span_wide(),
                                        arg_spans: args.iter().map(|arg| arg.error_span_wide()).collect(),
                                    });
                                }

                                else {
                                    for i in 0..params.len() {
                                        if let Err(()) = self.solve_supertype(
                                            &params[i],
                                            &arg_types[i],
                                            types,
                                            generic_args,
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

    pub fn get_type_of_field(
        &mut self,
        r#type: &Type,
        field: &[Field],
        types: &mut HashMap<Span, Type>,
        generic_args: &mut HashMap<(Span, Span), Type>,
    ) -> Result<Type, ()> {
        match r#type {
            Type::Static { def_span, .. } => {
                let mut field_type = None;

                match &field[0] {
                    Field::Name { name, name_span, .. } => match self.struct_shapes.get(def_span) {
                        Some(s) => {
                            for field in s.fields.iter() {
                                if field.name == *name {
                                    match types.get(&field.name_span) {
                                        Some(t) => {
                                            field_type = Some(t.clone());
                                        },
                                        None => {
                                            field_type = Some(Type::Var { def_span: field.name_span, is_return: false });
                                        },
                                    }

                                    break;
                                }
                            }

                            match s.associated_lets.get(name) {
                                Some(associated_let) => match types.get(associated_let) {
                                    Some(t) => {
                                        field_type = Some(t.clone());
                                    },
                                    None => {
                                        field_type = Some(Type::Var { def_span: *associated_let, is_return: false });
                                    },
                                },
                                None => match s.associated_funcs.get(name) {
                                    // `x.unwrap()` is desugared to `@associated_func_unwrap_1(x)`.
                                    // `@associated_func_unwrap_1` is a poly-generic function and we can
                                    // easily reference the function with its name.
                                    Some((params, is_pure, _)) => {
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
                                                    },
                                                }
                                            ).collect(),
                                            r#return: Box::new(Type::GenericArg {
                                                call: *name_span,
                                                generic: Span::Poly {
                                                    name: poly_name,
                                                    kind: PolySpanKind::Return,
                                                },
                                            }),
                                            purity: if *is_pure {
                                                FuncPurity::Pure
                                            } else {
                                                FuncPurity::Impure
                                            },
                                        });
                                    },
                                    None => {},
                                },
                            }
                        },
                        _ => {},
                    },
                    _ => todo!(),
                }

                match field_type {
                    Some(field_type) => {
                        if field.len() == 1 {
                            Ok(field_type)
                        }

                        else {
                            self.get_type_of_field(
                                &field_type,
                                &field[1..],
                                types,
                                generic_args,
                            )
                        }
                    },
                    None => {
                        // an error
                        todo!()
                    },
                }
            },
            Type::Tuple { args, .. } => {
                let mut field_type = None;

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
                    Field::Index(i) => todo!(),
                    Field::Range(start, end) => todo!(),
                    Field::Variant => todo!(),
                    Field::Constructor | Field::Payload => unreachable!(),
                };

                match field_type {
                    Some(field_type) => {
                        if field.len() == 1 {
                            Ok(field_type)
                        }

                        else {
                            self.get_type_of_field(
                                &field_type,
                                &field[1..],
                                types,
                                generic_args,
                            )
                        }
                    },
                    None => {
                        // maybe it's an associated function!
                        todo!()
                    },
                }
            },
            Type::Param { .. } => todo!(),
            // `Type::Blocked` exists exactly for this reason.
            // Read the documentation at `crates/mir/src/type.rs`.
            Type::Var { def_span: span, .. } |
            Type::GenericArg { call: span, .. } |
            Type::Blocked { origin: span } => {
                self.blocked_type_vars.insert(*span);
                Ok(Type::Blocked { origin: *span })
            },
            _ => panic!("TODO: {type:?}"),
        }
    }
}
