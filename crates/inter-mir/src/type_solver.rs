use crate::{Session, Type, write_log};
use crate::error::{ErrorContext, TypeError};
use sodigy_hir::FuncPurity;
use sodigy_mir::TypeAssertion;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::hash_map::Entry;

#[cfg(feature = "log")]
use crate::LogEntry;

mod assert;
mod expr;
mod func;
mod r#let;
mod pattern;

impl Session {
    pub fn apply_never_types(&mut self) {
        let mut never_types = vec![];

        for type_var in self.type_vars.keys() {
            match type_var {
                Type::Var { def_span, .. } => match self.types.get(def_span) {
                    None | Some(Type::Var { .. } | Type::GenericArg { .. }) => {
                        if let Some(never_type) = self.maybe_never_type.get(type_var) {
                            never_types.push((type_var.clone(), never_type.clone()));
                        }
                    },
                    _ => {},
                },
                Type::GenericArg { call, generic } => match self.generic_args.get(&(call.clone(), generic.clone())) {
                    None | Some(Type::Var { .. } | Type::GenericArg { .. }) => {
                        if let Some(never_type) = self.maybe_never_type.get(type_var) {
                            never_types.push((type_var.clone(), never_type.clone()));
                        }
                    },
                    _ => {},
                },
                _ => unreachable!(),
            }
        }

        for (type_var, never_type) in never_types.iter() {
            match type_var {
                Type::Var { def_span, is_return } => {
                    if *is_return {
                        match self.types.get_mut(def_span) {
                            Some(Type::Func { r#return, .. }) => {
                                **r#return = never_type.clone();
                            },
                            _ => unreachable!(),
                        }
                    }

                    else {
                        self.types.insert(def_span.clone(), never_type.clone());
                    }

                    self.substitute(type_var, never_type);
                },
                Type::GenericArg { call, generic } => {
                    self.generic_args.insert((call.clone(), generic.clone()), never_type.clone());
                    self.substitute(type_var, never_type);
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn check_all_types_infered(&mut self) -> Result<(), ()> {
        let mut has_error = false;

        for (type_var, id) in self.type_vars.iter() {
            match type_var {
                Type::Var { def_span, is_return } => match self.types.get(def_span) {
                    None | Some(Type::Var { .. } | Type::GenericArg { .. }) => {
                        if self.pattern_name_bindings.contains(def_span) {
                            continue;
                        }

                        has_error = true;
                        self.type_errors.push(TypeError::CannotInferType {
                            id: *id,
                            span: def_span.clone(),
                            is_return: false,
                        });
                    },
                    Some(t) => {
                        let type_vars = t.get_type_vars();

                        if !type_vars.is_empty() {
                            if *is_return {
                                let Type::Func { r#return: return_type, .. } = t else { unreachable!() };
                                let return_type = *return_type.clone();

                                match return_type {
                                    Type::Var { .. } | Type::GenericArg { .. } => {
                                        has_error = true;
                                        self.type_errors.push(TypeError::CannotInferType {
                                            id: *id,
                                            span: def_span.clone(),
                                            is_return: true,
                                        });
                                    },
                                    _ => {
                                        let type_vars = return_type.get_type_vars();

                                        if !type_vars.is_empty() {
                                            has_error = true;
                                            self.type_errors.push(TypeError::PartiallyInferedType {
                                                id: *id,
                                                span: def_span.clone(),
                                                r#type: return_type,
                                                is_return: true,
                                            });
                                        }
                                    },
                                }
                            }

                            else {
                                has_error = true;
                                self.type_errors.push(TypeError::PartiallyInferedType {
                                    id: *id,
                                    span: def_span.clone(),
                                    r#type: t.clone(),
                                    is_return: false,
                                });
                            }
                        }
                    },
                },
                Type::GenericArg { call, generic } => {
                    if self.solved_generic_args.contains(&(call.clone(), generic.clone())) {
                        continue;
                    }

                    match self.generic_args.get(&(call.clone(), generic.clone())) {
                        None | Some(Type::Var { .. } | Type::GenericArg { .. }) => {
                            has_error = true;
                            self.type_errors.push(TypeError::CannotInferGenericType {
                                call: call.clone(),
                                generic: generic.clone(),
                                func_def: self.generic_def_span_rev.get(generic).cloned(),
                            });
                        },
                        Some(t) => {
                            let type_vars = t.get_type_vars();

                            if !type_vars.is_empty() {
                                has_error = true;
                                self.type_errors.push(TypeError::PartiallyInferedGenericType {
                                    call: call.clone(),
                                    generic: generic.clone(),
                                    func_def: self.generic_def_span_rev.get(generic).cloned(),
                                    r#type: t.clone(),
                                });
                            }
                        },
                    }
                },
                _ => unreachable!(),
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn check_type_assertions(&mut self, type_assertions: &[TypeAssertion]) -> Result<(), ()> {
        let mut has_error = false;

        // We assume that `solve_supertype` doesn't affect each other
        // because all the type variables are already solved!
        for type_assertion in type_assertions.iter() {
            match self.types.get(&type_assertion.name_span) {
                Some(solved_type) => {
                    let solved_type = solved_type.clone();

                    if let Err(()) = self.solve_supertype(
                        &type_assertion.r#type,
                        &solved_type,
                        false,
                        Some(&type_assertion.type_span),
                        Some(&type_assertion.name_span),
                        ErrorContext::TypeAssertion,
                        false,
                    ) {
                        has_error = true;
                    }
                },
                None => unreachable!(),
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn add_type_var(&mut self, type_var: Type, id: Option<InternedString>) {
        match self.type_vars.entry(type_var) {
            Entry::Occupied(mut e) if id.is_some() => {
                *e.get_mut() = id;
            },
            Entry::Vacant(e) => {
                e.insert(id);
            },
            _ => {},
        }
    }

    pub fn add_type_var_ref(&mut self, reference: Type, referent: Type) {
        if reference != referent {
            match self.type_var_refs.entry(reference) {
                Entry::Occupied(mut e) => {
                    let refs = e.get_mut();

                    // It's O(n), but `n` is very small.
                    if !refs.contains(&referent) {
                        refs.push(referent);
                    }
                },
                Entry::Vacant(e) => {
                    e.insert(vec![referent]);
                },
            }
        }
    }

    /// It checks whether `lhs` is supertype of `rhs`. If so, it returns the supertype (`lhs`).
    /// For example, if there's Sodigy code `let x: Foo = y`, the compiler will call
    /// `solve_supertype(Foo, solve_expr(y))` because the type annotation has to be the supertype
    /// of the value.
    ///
    /// If `bidirectional` is set, it also checks if `rhs` is supertype of `lhs`. If either is a supertype
    /// of another, it returns the supertype.
    ///
    /// If the check fails, that's a type error. So, this function is a type-checker.
    /// If `lhs` or `rhs` has type variables, it tries to unify the variables. So, this
    /// function is a type-inferencer.
    ///
    /// Sodigy has very limited kinds of subtyping:
    /// 1. `Never` type is subtype of every other type.
    /// 2. `PureFn` is subtype of `Fn` and `ImpureFn` is subtype of `Fn`.
    /// 3. If type A and type B are exactly the same, A and B are subtype of each other.
    /// 4. Otherwise, it's a type error.
    pub fn solve_supertype(
        &mut self,
        lhs: &Type,
        rhs: &Type,

        // If it's checking a type argument (`Int` in `Option<Int>`), it doesn't
        // generate an error message, and its caller will.
        is_checking_argument: bool,

        // for helpful error messages
        lhs_span: Option<&Span>,
        rhs_span: Option<&Span>,
        context: ErrorContext,
        bidirectional: bool,
    ) -> Result<Type, ()> {
        write_log!(self, LogEntry::SolveSupertype {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
            lhs_span: lhs_span.cloned(),
            rhs_span: rhs_span.cloned(),
            context: context.clone(),
        });

        match (lhs, rhs) {
            (Type::Never(_), Type::Never(_)) => Ok(lhs.clone()),
            (
                Type::Data { constructor_def_span: constructor1, args: args1, .. },
                Type::Data { constructor_def_span: constructor2, args: args2, .. },
            ) => {
                if *constructor1 != *constructor2 {
                    if !is_checking_argument {
                        self.type_errors.push(TypeError::UnexpectedType {
                            expected: lhs.clone(),
                            expected_span: lhs_span.cloned(),
                            got: rhs.clone(),
                            got_span: rhs_span.cloned(),
                            context: context.clone(),
                        });
                    }

                    return Err(());
                }

                match (args1, args2) {
                    (Some(args1), Some(args2)) => {
                        if args1.len() != args2.len() {
                            if !is_checking_argument {
                                self.type_errors.push(TypeError::UnexpectedType {
                                    expected: lhs.clone(),
                                    expected_span: lhs_span.cloned(),
                                    got: rhs.clone(),
                                    got_span: rhs_span.cloned(),
                                    context: context.clone(),
                                });
                            }

                            Err(())
                        }

                        else {
                            let mut has_error = false;
                            let mut args = Vec::with_capacity(args1.len());

                            for i in 0..args1.len() {
                                match self.solve_supertype(
                                    &args1[i],
                                    &args2[i],
                                    true,  // is_checking_argument
                                    None,
                                    None,
                                    ErrorContext::None,
                                    bidirectional,
                                ) {
                                    Ok(arg) => {
                                        args.push(arg);
                                    },
                                    Err(()) => {
                                        if !is_checking_argument {
                                            self.type_errors.push(TypeError::UnexpectedType {
                                                expected: lhs.clone(),
                                                expected_span: lhs_span.cloned(),
                                                got: rhs.clone(),
                                                got_span: rhs_span.cloned(),
                                                context: context.clone(),
                                            });
                                        }
                                        has_error = true;
                                    },
                                }
                            }

                            if has_error {
                                Err(())
                            }

                            else {
                                Ok(Type::Data {
                                    constructor_def_span: constructor1.clone(),
                                    constructor_span: Span::None,
                                    args: Some(args),
                                    group_span: Some(Span::None),
                                })
                            }
                        }
                    },
                    (None, None) => Ok(lhs.clone()),
                    _ => Err(()),
                }
            },
            (Type::Func { r#return: return1, params: args1, purity: p1, .. }, Type::Func { r#return: return2, params: args2, purity: p2, .. }) => {
                let r#return = match self.solve_supertype(
                    return1,
                    return2,
                    true,  // is_checking_argument
                    None,
                    None,
                    context.clone(),
                    bidirectional,
                ) {
                    Ok(t) => t,
                    Err(()) => {
                        if !is_checking_argument {
                            self.type_errors.push(TypeError::UnexpectedType {
                                expected: lhs.clone(),
                                expected_span: lhs_span.cloned(),
                                got: rhs.clone(),
                                got_span: rhs_span.cloned(),
                                context: context.clone(),
                            });
                        }

                        return Err(());
                    },
                };

                if args1.len() != args2.len() {
                    if !is_checking_argument {
                        self.type_errors.push(TypeError::UnexpectedType {
                            expected: lhs.clone(),
                            expected_span: lhs_span.cloned(),
                            got: rhs.clone(),
                            got_span: rhs_span.cloned(),
                            context: context.clone(),
                        });
                    }

                    Err(())
                }

                else {
                    let mut has_error = false;
                    let mut args = Vec::with_capacity(args1.len());

                    for i in 0..args1.len() {
                        // TOOD: For function parameters, we need `solve_subtype`, but we don't have such.
                        //       So, 1) we swap `args1[i]` and `args2[i]` and 2) discard the result (which is the supertype)
                        //       and push `args1[i]` (which is the subtype) to `args`.
                        match self.solve_supertype(
                            &args2[i],
                            &args1[i],
                            true,  // is_checking_argument
                            None,
                            None,
                            ErrorContext::None,
                            bidirectional,
                        ) {
                            Ok(arg) => {
                                args.push(args1[i].clone());
                            },
                            Err(()) => {
                                if !is_checking_argument {
                                    self.type_errors.push(TypeError::UnexpectedType {
                                        expected: lhs.clone(),
                                        expected_span: lhs_span.cloned(),
                                        got: rhs.clone(),
                                        got_span: rhs_span.cloned(),
                                        context: context.clone(),
                                    });
                                }
                                has_error = true;
                            },
                        }
                    }

                    if has_error {
                        Err(())
                    }

                    else {
                        let purity = match (p1, p2) {
                            (FuncPurity::Both, _) => FuncPurity::Both,
                            (FuncPurity::Pure, FuncPurity::Pure) => FuncPurity::Pure,
                            (FuncPurity::Impure, FuncPurity::Impure) => FuncPurity::Impure,
                            _ => {
                                if bidirectional {
                                    FuncPurity::Both
                                }

                                else {
                                    if !is_checking_argument {
                                        self.type_errors.push(TypeError::UnexpectedPurity {
                                            expected_type: lhs.clone(),
                                            expected_purity: *p1,
                                            expected_span: lhs_span.cloned(),
                                            got_type: rhs.clone(),
                                            got_purity: *p2,
                                            got_span: rhs_span.cloned(),
                                        });
                                    }

                                    return Err(());
                                }
                            },
                        };

                        Ok(Type::Func {
                            fn_span: Span::None,
                            group_span: Span::None,
                            params: args,
                            r#return: Box::new(r#return),
                            purity,
                        })
                    }
                }
            },
            (
                t1 @ Type::Var { def_span: v1, is_return: is_return1 },
                t2 @ Type::Var { def_span: v2, is_return: is_return2 },
            ) => {
                if *v1 == *v2 {
                    Ok(lhs.clone())
                }

                else {
                    let maybe_solved_t1 = match self.types.get(v1) {
                        Some(Type::Func { r#return, .. }) if *is_return1 => r#return,
                        Some(t) => t,
                        _ => t1,
                    };
                    let maybe_solved_t2 = match self.types.get(v2) {
                        Some(Type::Func { r#return, .. }) if *is_return2 => r#return,
                        Some(t) => t,
                        _ => t2,
                    };

                    match (maybe_solved_t1, maybe_solved_t2) {
                        (
                            Type::Var { .. } | Type::GenericArg { .. },
                            Type::Var { .. } | Type::GenericArg { .. },
                        ) => {},
                        (c1, c2) => {
                            let c1 = c1.clone();
                            let c2 = c2.clone();
                            return self.solve_supertype(
                                &c1,
                                &c2,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                    }


                    if *is_return1 {
                        match self.types.get_mut(v1) {
                            Some(Type::Func { r#return, .. }) => {
                                **r#return = t2.clone();
                            },
                            _ => unreachable!(),
                        }
                    } else {
                        self.types.insert(v1.clone(), t2.clone());
                    }

                    self.add_type_var(t1.clone(), None);
                    self.add_type_var_ref(t1.clone(), t2.clone());

                    if *is_return2 {
                        match self.types.get_mut(v2) {
                            Some(Type::Func { r#return, .. }) => {
                                **r#return = t1.clone();
                            },
                            _ => unreachable!(),
                        }
                    } else {
                        self.types.insert(v2.clone(), t1.clone());
                    }

                    self.add_type_var(t2.clone(), None);
                    self.add_type_var_ref(t2.clone(), t1.clone());
                    Ok(t1.clone())
                }
            },
            (t1 @ Type::GenericArg { call: c1, generic: g1 }, t2 @ Type::GenericArg { call: c2, generic: g2 }) => {
                if c1 == c2 && g1 == g2 {
                    Ok(lhs.clone())
                }

                else {
                    match self.generic_args.get(&(c1.clone(), g1.clone())) {
                        Some(Type::Var { .. } | Type::GenericArg { .. }) => {},
                        Some(type1) => {
                            let type1 = type1.clone();
                            return self.solve_supertype(
                                &type1,
                                t2,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                        None => {},
                    }

                    match self.generic_args.get(&(c2.clone(), g2.clone())) {
                        Some(Type::Var { .. } | Type::GenericArg { .. }) => {},
                        Some(type2) => {
                            let type2 = type2.clone();
                            return self.solve_supertype(
                                t1,
                                &type2,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                        None => {},
                    }

                    self.generic_args.insert((c1.clone(), g1.clone()), t2.clone());
                    self.add_type_var(t1.clone(), None);
                    self.add_type_var_ref(t1.clone(), t2.clone());
                    self.generic_args.insert((c2.clone(), g2.clone()), t1.clone());
                    self.add_type_var(t2.clone(), None);
                    self.add_type_var_ref(t2.clone(), t1.clone());
                    Ok(t1.clone())
                }
            },
            (Type::Blocked { .. }, t) | (t, Type::Blocked { .. }) => Ok(t.clone()),
            (Type::GenericParam { .. }, _) | (_, Type::GenericParam { .. }) => {
                // We'll only type check/infer monomorphized functions.
                unreachable!()
            },
            (never @ Type::Never(_), concrete) | (concrete, never @ Type::Never(_)) => {
                let never_type_expected = matches!(lhs, Type::Never(_));

                // We don't solve the variable, because we might solve it with a more concrete type.
                // But we still have to remember that this variable might be `Type::Never`.
                // If we can't solve the variable, we'll assign `Type::Never` to the variable.
                match concrete {
                    Type::Var { .. } | Type::GenericArg { .. } => {
                        self.maybe_never_type.insert(concrete.clone(), never.clone());
                    },
                    _ => {},
                }

                // `Type::Never` is subtype of every type, but `concrete` is not a
                // subtype of `Type::Never`.
                if bidirectional || !never_type_expected {
                    Ok(concrete.clone())
                } else {
                    self.type_errors.push(TypeError::UnexpectedType {
                        expected: lhs.clone(),
                        expected_span: lhs_span.cloned(),
                        got: rhs.clone(),
                        got_span: rhs_span.cloned(),
                        context: context.clone(),
                    });
                    Err(())
                }
            },
            (
                type_var @ Type::Var { def_span, is_return },
                maybe_concrete @ (Type::Data { .. } | Type::Func { .. }),
            ) | (
                maybe_concrete @ (Type::Data { .. } | Type::Func { .. }),
                type_var @ Type::Var { def_span, is_return },
            ) => {
                let ref_type_vars = maybe_concrete.get_type_vars();
                let concrete_span = if let Type::Var { .. } = lhs {
                    rhs_span
                } else {
                    lhs_span
                };

                if *is_return {
                    // If previously infered type and newly infered type are different,
                    // that's an error!
                    match self.types.get(def_span) {
                        Some(Type::Func { r#return, .. }) => match &**r#return {
                            Type::Var { .. } | Type::GenericArg { .. } => {},
                            prev_infered => {
                                let prev_infered = prev_infered.clone();

                                if let Err(()) = self.solve_supertype(
                                    &prev_infered,
                                    maybe_concrete,
                                    false,
                                    None,
                                    concrete_span,
                                    ErrorContext::InferedAgain { type_var: type_var.clone() },
                                    bidirectional,
                                ) {
                                    return Err(());
                                }
                            },
                        },
                        _ => unreachable!(),
                    }

                    match self.types.get_mut(def_span) {
                        Some(Type::Func { r#return, .. }) => {
                            **r#return = maybe_concrete.clone();
                        },
                        _ => unreachable!(),
                    }
                }

                else {
                    // If previously infered type and newly infered type are different,
                    // that's an error!
                    match self.types.get(def_span) {
                        Some(Type::Var { .. } | Type::GenericArg { .. }) => {},
                        Some(prev_infered) => {
                            let prev_infered = prev_infered.clone();

                            if let Err(()) = self.solve_supertype(
                                &prev_infered,
                                maybe_concrete,
                                false,
                                None,
                                concrete_span,
                                ErrorContext::InferedAgain { type_var: type_var.clone() },
                                bidirectional,
                            ) {
                                return Err(());
                            }
                        },
                        None => {},
                    }

                    self.types.insert(def_span.clone(), maybe_concrete.clone());
                }

                if ref_type_vars.is_empty() {
                    self.substitute(type_var, maybe_concrete);
                }

                else {
                    for ref_type_var in ref_type_vars.into_iter() {
                        self.add_type_var_ref(ref_type_var, type_var.clone());
                    }
                }

                Ok(maybe_concrete.clone())
            },
            (
                type_var @ Type::GenericArg { call, generic },
                maybe_concrete @ (Type::Data { .. } | Type::Func { .. }),
            ) | (
                maybe_concrete @ (Type::Data { .. } | Type::Func { .. }),
                type_var @ Type::GenericArg { call, generic },
            ) => {
                let ref_type_vars = maybe_concrete.get_type_vars();
                let concrete_span = if let Type::Var { .. } = lhs {
                    rhs_span
                } else {
                    lhs_span
                };

                match self.generic_args.get(&(call.clone(), generic.clone())) {
                    Some(Type::Var { .. } | Type::GenericArg { .. }) => {},
                    Some(prev_infered) => {
                        let prev_infered = prev_infered.clone();

                        if let Err(()) = self.solve_supertype(
                            &prev_infered,
                            maybe_concrete,
                            false,
                            None,
                            concrete_span,
                            ErrorContext::InferedAgain { type_var: type_var.clone() },
                            bidirectional,
                        ) {
                            return Err(());
                        }
                    },
                    None => {},
                }

                self.generic_args.insert((call.clone(), generic.clone()), maybe_concrete.clone());

                if ref_type_vars.is_empty() {
                    self.substitute(type_var, maybe_concrete);
                }

                else {
                    for ref_type_var in ref_type_vars.into_iter() {
                        self.add_type_var_ref(ref_type_var, type_var.clone());
                    }
                }

                Ok(maybe_concrete.clone())
            },
            (Type::Data { .. } | Type::Func { .. }, Type::Data { .. } | Type::Func { .. }) => {
                if !is_checking_argument {
                    self.type_errors.push(TypeError::UnexpectedType {
                        expected: lhs.clone(),
                        expected_span: lhs_span.cloned(),
                        got: rhs.clone(),
                        got_span: rhs_span.cloned(),
                        context,
                    });
                }

                Err(())
            },
            (
                tv @ Type::Var { def_span, is_return },
                gi @ Type::GenericArg { call, generic },
            ) | (
                gi @ Type::GenericArg { call, generic },
                tv @ Type::Var { def_span, is_return },
            ) => {
                let (tv_span, gi_span) = if let Type::Var { .. } = lhs {
                    (lhs_span, rhs_span)
                } else {
                    (rhs_span, lhs_span)
                };

                match self.types.get(def_span) {
                    Some(Type::Var { .. } | Type::GenericArg { .. }) => {},
                    Some(tv_concrete) => {
                        // `fn my_add(a, b) = foo(a, b); fn foo<T, U, V>(a: T, b: U) -> V;`
                        // def_span: `my_add`
                        // call: `foo` in `foo(a, b)`
                        // generic: `V`
                        // types.get(def_span): `Some(Fn(?x1, ?x2) -> ?x3)`
                        // We have to solve `?x3 = gi`, so we have to extract `?x3` from `tv_concrete`.
                        if *is_return {
                            match tv_concrete {
                                Type::Func { r#return, .. } => match &**r#return {
                                    Type::Var { .. } | Type::GenericArg { .. } => {},
                                    tv_concrete => {
                                        let tv_concrete = tv_concrete.clone();
                                        return self.solve_supertype(
                                            &tv_concrete,
                                            gi,
                                            is_checking_argument,
                                            tv_span,
                                            gi_span,
                                            ErrorContext::Deep,
                                            bidirectional,
                                        );
                                    },
                                },
                                _ => unreachable!(),
                            }
                        }

                        else {
                            let tv_concrete = tv_concrete.clone();
                            return self.solve_supertype(
                                &tv_concrete,
                                gi,
                                is_checking_argument,
                                tv_span,
                                gi_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        }
                    },
                    None => {},
                }

                // TODO: I want to `match generic_args.get(&(call.clone(), generic.clone()))`, but it's
                //       complicated due to the `is_return` field...

                if !*is_return {
                    self.types.insert(def_span.clone(), gi.clone());
                    self.add_type_var(tv.clone(), None);
                    self.add_type_var_ref(tv.clone(), gi.clone());
                    self.generic_args.insert((call.clone(), generic.clone()), tv.clone());
                    self.add_type_var(gi.clone(), None);
                    self.add_type_var_ref(gi.clone(), tv.clone());
                    Ok(tv.clone())
                }

                else {
                    // TODO: I want to create more type expressions here, but it's complicated
                    //       due to the `is_return` field...
                    Ok(lhs.clone())
                }
            },
        }
    }

    // Let's say there's a type expression: `Type::Var(x) = Type::Param { unit, args: [Type::Static(Int), Type::Var(y)] }`.
    // When we infered that `Type::Var(y) = Type::Static(Int)`, we have to update `Type::Var(x)`.
    // In this case, we call `self.substitute(y, Int)`.
    // The relationship between `x` and `y` are stored in `self.type_var_refs`.
    fn substitute(&mut self, type_var: &Type, r#type: &Type) {
        let ref_types = self.type_var_refs.get(type_var).map(|refs| refs.to_vec()).unwrap_or(vec![]);
        let mut newly_completed_type_vars = vec![];

        for ref_type_var in ref_types.iter() {
            match ref_type_var {
                Type::Var { def_span, is_return } => match self.types.get_mut(def_span) {
                    Some(ref_type) => {
                        if *is_return {
                            match ref_type {
                                Type::Func { r#return, .. } => {
                                    r#return.substitute(type_var, r#type);
                                },
                                _ => unreachable!(),
                            }
                        }

                        else {
                            ref_type.substitute(type_var, r#type);
                        }

                        if ref_type.get_type_vars().is_empty() {
                            newly_completed_type_vars.push(ref_type_var);
                        }
                    },
                    None => unreachable!(),
                },
                Type::GenericArg { call, generic } => match self.generic_args.get_mut(&(call.clone(), generic.clone())) {
                    Some(ref_type) => {
                        ref_type.substitute(type_var, r#type);

                        if ref_type.get_type_vars().is_empty() {
                            newly_completed_type_vars.push(ref_type_var);
                        }
                    },
                    None => unreachable!(),
                },
                _ => unreachable!(),
            }
        }

        self.type_var_refs.remove(type_var);

        for type_var in newly_completed_type_vars.iter() {
            match type_var {
                Type::Var { def_span, is_return } => {
                    let r#type: Type = match self.types.get(def_span) {
                        Some(Type::Func { r#return, .. }) if *is_return => r#return.as_ref().clone(),
                        Some(t) if !*is_return => t.clone(),
                        _ => unreachable!(),
                    };

                    self.substitute(type_var, &r#type);
                },
                Type::GenericArg { call, generic } => {
                    let r#type = self.generic_args.get_mut(&(call.clone(), generic.clone())).unwrap().clone();
                    self.substitute(type_var, &r#type);
                },
                _ => unreachable!(),
            }
        }
    }
}
