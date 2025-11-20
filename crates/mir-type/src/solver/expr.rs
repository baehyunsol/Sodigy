use super::Solver;
use crate::{Expr, Type};
use crate::error::{ErrorContext, TypeError};
use sodigy_mir::{Callable, ShortCircuitKind};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    // FIXME: there are A LOT OF heap allocations
    //
    // It can solve type of any expression, but the result maybe `Type::Var`.
    // If it finds new type equations while solving, it adds them to `type_equations`.
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
            Expr::Identifier(id) => match types.get(&id.def_span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    match id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                            // `False` in `Bool.False` has type `Bool`.
                            NameKind::EnumVariant { parent } => {
                                return (Some(Type::Static(parent)), false);
                            },
                            _ => panic!("{id:?}"),
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
                true => (Some(Type::Static(self.get_lang_item_span("type.Int"))), false),
                false => (Some(Type::Static(self.get_lang_item_span("type.Number"))), false),
            },
            Expr::String { binary, .. } => match *binary {
                true => (
                    Some(Type::Param {
                        r#type: Box::new(Type::Static(self.get_lang_item_span("type.List"))),
                        args: vec![Type::Static(self.get_lang_item_span("type.Byte"))],
                        group_span: Span::None,
                    }),
                    false,
                ),
                false => (
                    Some(Type::Param {
                        r#type: Box::new(Type::Static(self.get_lang_item_span("type.List"))),
                        args: vec![Type::Static(self.get_lang_item_span("type.Char"))],
                        group_span: Span::None,
                    }),
                    false,
                ),
            },
            Expr::If(r#if) => {
                let (cond_type, mut has_error) = self.solve_expr(r#if.cond.as_ref(), types, generic_instances);
                if let Some(cond_type) = cond_type {
                    if let Err(()) = self.solve_subtype(
                        &Type::Static(self.get_lang_item_span("type.Bool")),
                        &cond_type,
                        types,
                        generic_instances,
                        false,
                        None,
                        Some(r#if.cond.error_span()),
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
                        Some(r#if.true_value.error_span()),
                        Some(r#if.false_value.error_span()),
                        ErrorContext::IfValueEqual,
                    ) {
                        Ok(expr_type) => (Some(expr_type), has_error | e1 | e2),
                        Err(()) => (None, true),
                    },
                    _ => (None, true),
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
            Expr::FieldModifier { fields, lhs, rhs } => todo!(),
            Expr::ShortCircuit { lhs, rhs, kind, .. } => {
                let bool_type = Type::Static(self.get_lang_item_span("type.Bool"));
                let context = match kind {
                    ShortCircuitKind::And => ErrorContext::ShortCircuitAndBool,
                    ShortCircuitKind::Or => ErrorContext::ShortCircuitOrBool,
                };
                let mut has_error = false;
                let (lhs_type, e1) = self.solve_expr(lhs.as_ref(), types, generic_instances);
                has_error |= e1;
                let (rhs_type, e2) = self.solve_expr(rhs.as_ref(), types, generic_instances);
                has_error |= e2;

                if let Some(lhs_type) = lhs_type {
                    if let Err(()) = self.solve_subtype(
                        &bool_type,
                        &lhs_type,
                        types,
                        generic_instances,
                        false,
                        None,
                        Some(lhs.error_span()),
                        context,
                    ) {
                        has_error = true;
                    }
                }

                if let Some(rhs_type) = rhs_type {
                    if let Err(()) = self.solve_subtype(
                        &bool_type,
                        &rhs_type,
                        types,
                        generic_instances,
                        false,
                        None,
                        Some(rhs.error_span()),
                        context,
                    ) {
                        has_error = true;
                    }
                }

                (Some(bool_type), has_error)
            },
            // 1. we can solve types of args
            // 2. if callable is...
            //    - a function without generic
            //      - every arg must have a concrete type, so does the return type
            //      - it calls `equal` for all args, and returns the return type
            //    - a generic function
            //      - it first converts `Generic` to `GenericInstance` and does what
            //        a non-generic function does
            Expr::Call { func, args, generic_defs, given_keyword_arguments } => {
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
                            args: arg_defs,
                            r#return,
                            ..
                        }) => {
                            let mut arg_defs = arg_defs.clone();
                            let mut return_type: Type = *r#return.clone();
                            let span = *span;

                            if !generic_defs.is_empty() {
                                for arg_def in arg_defs.iter_mut() {
                                    arg_def.substitute_generic_def(span, &generic_defs);
                                }

                                return_type.substitute_generic_def(span, &generic_defs);

                                for generic_def in generic_defs.iter() {
                                    self.add_type_var(Type::GenericInstance { call: span, generic: *generic_def }, None);
                                }
                            }

                            // It doesn't check arg types if there are wrong number of args.
                            // Whether or not there're type errors with args, it returns the return type.
                            if arg_types.len() != arg_defs.len() {
                                has_error = true;
                                self.errors.push(TypeError::WrongNumberOfArguments {
                                    expected: arg_defs,
                                    got: arg_types,
                                    given_keyword_arguments: given_keyword_arguments.to_vec(),
                                    func_span: func.error_span(),
                                    arg_spans: args.iter().map(|arg| arg.error_span()).collect(),
                                });
                            }

                            else {
                                for (i, arg_def) in arg_defs.iter().enumerate() {
                                    if let Err(()) = self.solve_subtype(
                                        arg_def,
                                        &arg_types[i],
                                        types,
                                        generic_instances,
                                        false,
                                        None,
                                        Some(args[i].error_span()),
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
                        Some(t @ (Type::Static(_) | Type::Unit(_) | Type::Param { .. })) => {
                            self.errors.push(TypeError::NotCallable {
                                r#type: t.clone(),
                                func_span: *span,
                            });
                            (None, true)
                        },
                        // We only type check/infer monomorphized functions.
                        Some(Type::GenericDef(_)) => unreachable!(),
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
                            // and `Callable::TupleInit`'s `group_span` is of the expression/
                            r#type: Box::new(Type::Unit(Span::None)),
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
                                r#type: Box::new(Type::Static(self.get_lang_item_span("type.List"))),
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
                                    Some(args[0].error_span()),
                                    Some(args[i].error_span()),
                                    ErrorContext::ListElementEqual,
                                ) {
                                    elem_type = new_elem_type;
                                }

                                else {
                                    has_error = true;
                                }
                            }

                            let r#type = Type::Param {
                                r#type: Box::new(Type::Static(self.get_lang_item_span("type.List"))),
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
                            Type::Static(_) | Type::Unit(_) | Type::Param { .. } => {
                                self.errors.push(TypeError::NotCallable {
                                    r#type: func_type.clone(),
                                    func_span: func.error_span(),
                                });
                                return (None, true);
                            },

                            // We'll only type check/infer monomorphized functions.
                            Type::GenericDef(_) => unreachable!(),

                            Type::Func { args: arg_defs, r#return, .. } => {
                                // It doesn't check arg types if there are wrong number of args.
                                // Whether or not there're type errors with args, it returns the return type.
                                if arg_types.len() != arg_defs.len() {
                                    has_error = true;
                                    self.errors.push(TypeError::WrongNumberOfArguments {
                                        expected: arg_defs,
                                        got: arg_types,
                                        given_keyword_arguments: given_keyword_arguments.to_vec(),
                                        func_span: func.error_span(),
                                        arg_spans: args.iter().map(|arg| arg.error_span()).collect(),
                                    });
                                }

                                else {
                                    for i in 0..arg_defs.len() {
                                        if let Err(()) = self.solve_subtype(
                                            &arg_defs[i],
                                            &arg_types[i],
                                            types,
                                            generic_instances,
                                            false,
                                            None,
                                            Some(args[i].error_span()),
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
            _ => panic!("TODO: {expr:?}"),
        }
    }
}
