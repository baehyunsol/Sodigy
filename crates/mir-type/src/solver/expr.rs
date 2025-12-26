use super::Solver;
use crate::{Expr, Type};
use crate::error::{ErrorContext, TypeError};
use sodigy_mir::{Callable, ShortCircuitKind};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::intern_string;
use std::collections::HashMap;

impl Solver {
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
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
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
                            _ => panic!("TODO: {id:?}"),
                        },
                        _ => {},
                    }

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
            Expr::Number { n, .. } => match n.is_integer {
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
            Expr::String { binary, .. } => match *binary {
                true => (
                    Some(Type::Param {
                        constructor: Box::new(Type::Static {
                            def_span: self.get_lang_item_span("type.List"),
                            span: Span::None,
                        }),
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
                        constructor: Box::new(Type::Static {
                            def_span: self.get_lang_item_span("type.List"),
                            span: Span::None,
                        }),
                        args: vec![Type::Static {
                            def_span: self.get_lang_item_span("type.Char"),
                            span: Span::None,
                        }],
                        group_span: Span::None,
                    }),
                    false,
                ),
            },
            Expr::Char { .. } => (
                Some(Type::Static {
                    def_span: self.get_lang_item_span("type.Char"),
                    span: Span::None,
                }),
                false,
            ),
            Expr::Byte { .. } => (
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
                        let (v_type, e) = self.solve_expr(v, types, generic_instances);

                        if e {
                            has_error = true;
                        }

                        if let Some(v_type) = v_type {
                            if let Err(()) = self.solve_subtype(
                                &bool_type,
                                &v_type,
                                types,
                                generic_instances,
                                false,
                                None,
                                Some(v.error_span_wide()),
                                context.clone(),
                            ) {
                                has_error = true;
                            }
                        }
                    }

                    (Some(bool_type), has_error)
                },
                None => {
                    let (cond_type, mut has_error) = self.solve_expr(r#if.cond.as_ref(), types, generic_instances);

                    if let Some(cond_type) = cond_type {
                        if let Err(()) = self.solve_subtype(
                            &Type::Static {
                                def_span: self.get_lang_item_span("type.Bool"),
                                span: Span::None,
                            },
                            &cond_type,
                            types,
                            generic_instances,
                            false,
                            None,
                            Some(r#if.cond.error_span_wide()),
                            ErrorContext::IfConditionBool,
                        ) {
                            has_error = true;
                        }
                    }

                    match (
                        self.solve_expr(r#if.true_value.as_ref(), types, generic_instances),
                        self.solve_expr(r#if.false_value.as_ref(), types, generic_instances),
                    ) {
                        ((Some(true_type), e1), (Some(false_type), e2)) => match self.solve_subtype(
                            &true_type,
                            &false_type,
                            types,
                            generic_instances,
                            false,
                            Some(r#if.true_value.error_span_wide()),
                            Some(r#if.false_value.error_span_wide()),
                            ErrorContext::IfValueEqual,
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
            Expr::Match(r#match) => {
                let (scrutinee_type, mut has_error) = self.solve_expr(r#match.scrutinee.as_ref(), types, generic_instances);
                let mut arm_types = Vec::with_capacity(r#match.arms.len());

                // TODO: it's okay to fail to infer the types of name bindings
                //       we need some kinda skip list
                for arm in r#match.arms.iter() {
                    if let Some(scrutinee_type) = &scrutinee_type {
                        match self.solve_pattern(&arm.pattern, types, generic_instances) {
                            (Some(pattern_type), e) => {
                                if let Err(()) = self.solve_subtype(
                                    &scrutinee_type,
                                    &pattern_type,
                                    types,
                                    generic_instances,
                                    false,
                                    Some(r#match.scrutinee.error_span_wide()),
                                    Some(arm.pattern.error_span_wide()),
                                    ErrorContext::MatchScrutinee,
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
                        let (guard_type, e) = self.solve_expr(guard, types, generic_instances);
                        has_error |= e;

                        if let Some(guard_type) = guard_type {
                            if let Err(()) = self.solve_subtype(
                                &Type::Static {
                                    def_span: self.get_lang_item_span("type.Bool"),
                                    span: Span::None,
                                },
                                &guard_type,
                                types,
                                generic_instances,
                                false,
                                None,
                                Some(guard.error_span_wide()),
                                ErrorContext::MatchGuardBool,
                            ) {
                                has_error = true;
                            }
                        }
                    }

                    let (arm_type, e) = self.solve_expr(&arm.value, types, generic_instances);
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
                        if let Ok(new_expr_type) = self.solve_subtype(
                            &expr_type,
                            &arm_types[i],
                            types,
                            generic_instances,
                            false,
                            Some(r#match.arms[0].value.error_span_wide()),
                            Some(r#match.arms[i].value.error_span_wide()),
                            ErrorContext::MatchArmEqual,
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
                    let (_, e) = self.solve_let(r#let, types, generic_instances);
                    has_error |= e;
                }

                for assert in block.asserts.iter() {
                    if let Err(()) = self.solve_assert(assert, types, generic_instances) {
                        has_error = true;
                    }
                }

                let (expr_type, e) = self.solve_expr(block.value.as_ref(), types, generic_instances);
                (expr_type, e || has_error)
            },
            Expr::Path { lhs, fields } => match self.solve_expr(lhs, types, generic_instances) {
                (Some(lhs_type), has_error) => match self.get_type_of_field(&lhs_type, fields, types, generic_instances) {
                    Ok(lhs_type) => (Some(lhs_type), has_error),
                    Err(()) => (None, true),
                },
                (None, _) => (None, true),
            },
            // 1. Make sure that `lhs` has the fields.
            // 2. Make sure that the field's type and `rhs`' type are the same.
            // 3. Return the type of `lhs`.
            Expr::FieldModifier { fields, lhs, rhs } => todo!(),
            // 1. we can solve types of args
            // 2. if callable is...
            //    - a function without generic
            //      - every arg must have a concrete type, so does the return type
            //      - it calls `equal` for all args, and returns the return type
            //    - a generic function
            //      - it first converts `Generic` to `GenericInstance` and does what
            //        a non-generic function does
            Expr::Call { func, args, generic_defs, given_keyword_arguments, .. } => {
                let mut has_error = false;
                let mut arg_types = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    match self.solve_expr(arg, types, generic_instances) {
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
                            ..
                        }) => {
                            let mut params = params.clone();
                            let mut return_type: Type = *r#return.clone();
                            let span = *span;

                            if !generic_defs.is_empty() {
                                for param in params.iter_mut() {
                                    param.substitute_generic_def(span, &generic_defs);
                                }

                                return_type.substitute_generic_def(span, &generic_defs);

                                for generic_def in generic_defs.iter() {
                                    self.add_type_var(Type::GenericInstance { call: span, generic: *generic_def }, None);
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
                                    if let Err(()) = self.solve_subtype(
                                        param,
                                        &arg_types[i],
                                        types,
                                        generic_instances,
                                        false,
                                        None,
                                        Some(args[i].error_span_wide()),
                                        ErrorContext::FuncArgs,
                                    ) {
                                        has_error = true;
                                    }
                                }
                            }

                            (Some(return_type), has_error)
                        },
                        // We're sure that `f` is not a function.
                        // For example, `let f = 3; f()`.
                        Some(t @ (Type::Static { .. } | Type::Unit(_) | Type::Param { .. })) => {
                            self.errors.push(TypeError::NotCallable {
                                r#type: t.clone(),
                                func_span: *span,
                            });
                            (None, true)
                        },
                        // We only type check/infer monomorphized functions.
                        Some(Type::GenericDef { .. }) => unreachable!(),
                        // This is not a type error because `!` is subtype of every type.
                        Some(t @ Type::Never(_)) => (Some(t.clone()), has_error),
                        // `let foo = bar(); foo()`.
                        // We're solving the expression `foo()`, we don't know the exact type
                        // of `foo` and `bar()`, but we now know that they have the same type.
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => todo!(),
                        None => todo!(),
                    },
                    Callable::TupleInit { .. } => (
                        Some(Type::Param {
                            // `Type::Unit`'s `group_span` is of type annotation,
                            // and `Callable::TupleInit`'s `group_span` is of the expression.
                            constructor: Box::new(Type::Unit(Span::None)),
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
                            let type_var = Type::GenericInstance { call: *group_span, generic: self.get_lang_item_span("built_in.init_list.generic.0") };
                            self.add_type_var(type_var.clone(), None);

                            let r#type = Type::Param {
                                constructor: Box::new(Type::Static {
                                    def_span: self.get_lang_item_span("type.List"),
                                    span: Span::None,
                                }),
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
                                if let Ok(new_elem_type) = self.solve_subtype(
                                    &elem_type,
                                    &arg_types[i],
                                    types,
                                    generic_instances,
                                    false,
                                    Some(args[0].error_span_wide()),
                                    Some(args[i].error_span_wide()),
                                    ErrorContext::ListElementEqual,
                                ) {
                                    elem_type = new_elem_type;
                                }

                                else {
                                    has_error = true;
                                }
                            }

                            let r#type = Type::Param {
                                constructor: Box::new(Type::Static {
                                    def_span: self.get_lang_item_span("type.List"),
                                    span: Span::None,
                                }),
                                args: vec![elem_type],

                                // this is for the type annotation, hence None
                                group_span: Span::None,
                            };
                            (Some(r#type), has_error)
                        }
                    },
                    Callable::Dynamic(func) => {
                        let (func_type, mut has_error) = match self.solve_expr(func, types, generic_instances) {
                            (Some(func_type), has_error) => (func_type, has_error),
                            (None, has_error) => {
                                return (None, has_error);
                            },
                        };

                        match func_type {
                            // TODO: What if there's a callable `Type::Static()` or `Type::Param {}`?
                            Type::Static { .. } | Type::Unit(_) | Type::Param { .. } => {
                                self.errors.push(TypeError::NotCallable {
                                    r#type: func_type.clone(),
                                    func_span: func.error_span_wide(),
                                });
                                return (None, true);
                            },

                            // We'll only type check/infer monomorphized functions.
                            Type::GenericDef { .. } => unreachable!(),

                            Type::Func { params, r#return, .. } => {
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
                                        if let Err(()) = self.solve_subtype(
                                            &params[i],
                                            &arg_types[i],
                                            types,
                                            generic_instances,
                                            false,
                                            None,
                                            Some(args[i].error_span_wide()),
                                            ErrorContext::FuncArgs,
                                        ) {
                                            has_error = true;
                                        }
                                    }
                                }

                                (Some(*r#return.clone()), has_error)
                            },
                            _ => panic!("TODO: {func:?}"),
                        }
                    },
                    _ => panic!("TODO: {func:?}"),
                }
            },
        }
    }

    pub fn get_type_of_field(
        &mut self,
        r#type: &Type,
        field: &[Field],
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> Result<Type, ()> {
        match r#type {
            Type::Static { def_span, .. } => todo!(),
            Type::Unit(_) => todo!(),  // It must be an error... right?
            Type::Param { constructor, args, .. } => {
                if let Type::Unit(_) = &**constructor {
                    let field_type = match &field[0] {
                        Field::Name { name, .. } => {
                            let mut field_type = None;

                            for i in 0..args.len() {
                                let i_s = format!("_{i}");

                                if intern_string(i_s.as_bytes(), &self.intermediate_dir).unwrap() == *name {
                                    field_type = Some(args[i].clone());
                                    break;
                                }
                            }

                            match field_type {
                                Some(field_type) => field_type,
                                None => todo!(),  // error: no such field
                            }
                        },
                        Field::Index(i) => todo!(),
                        Field::Range(start, end) => todo!(),
                        Field::Constructor | Field::Payload => unreachable!(),
                    };

                    if field.len() == 1 {
                        Ok(field_type)
                    }

                    else {
                        self.get_type_of_field(
                            &field_type,
                            &field[1..],
                            types,
                            generic_instances,
                        )
                    }
                }

                else {
                    todo!()
                }
            },
            _ => todo!(),
        }
    }
}
