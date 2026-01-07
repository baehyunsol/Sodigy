use crate::Type;
use crate::error::{ErrorContext, TypeError, TypeWarning};
use sodigy_hir::FuncPurity;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

mod assert;
mod expr;
mod func;
mod r#let;
mod pattern;

// When a type-variable is solved, it removes an entry in `type_var_refs`, but
// not in `type_vars`, because
// 1. We'll later use `type_vars` to distinguish what're infered types and what're annotated types.
// 2. If we don't remove entries in `type_var_refs`, cyclic type vars will cause a stack overflow.
pub struct TypeSolver {
    // Whenever `types.get(span)` returns `None`, it creates a type variable
    // and inserts the `span` to this hash set. It's later used to check
    // if all the type variables are infered.
    //
    // If the type variable is from a type annotation and a name is bound to
    // the type annotation, it also collects the name: that'd be helpful when
    // creating error messages.
    //
    // The key (`Type`) is either `Type::Var` or `Type::GenericInstance`.
    // Every type variable the type-solver encountered must be in this map.
    // The value being `None` or `Some(_)`... doesn't mean much. It's just used to
    // help generating error messages. If you want to check if a variable has been
    // successfully infered, you have to check `types` or `generic_instances`, which
    // do not belong to `Solver`.
    pub type_vars: HashMap<Type, Option<InternedString>>,

    // If a type variable references another type variable, we have to track the relation.
    // For example, if a type of function `add` is `Type::Var(add) = Fn(Type::Var(x), Type::Var(y)) -> Int`,
    // we have to update `TypeVar(add)` when `TypeVar(x)` is updated. So, we `type_var_refs.get(x)`
    // will give you a vector with `add`.
    // If a type variable references itself, that should not be included in the Vec<Span>.
    //
    // A type var can be either `Type::Var` or `Type::GenericInstance`.
    pub type_var_refs: HashMap<Type, Vec<Type>>,

    // If it infers that `Type::Var(x) = Type::Never`, it doesn't substitute
    // `x` with `Type::Never` and continues to infer `x`.
    // For example, if `x` is infered to `Type::Never` and `Type::Static(Int)`, it
    // chooses `Type::Static(Int)` because `Type::Never` is subtype of `Type::Static(Int)`.
    // But if it cannot find any more information about `x`, it has to choose `Type::Never`.
    // So, after type inference is done, if there's an un-infered type variable and the variable
    // is in this set, the type variable has `Type::Never`.
    pub maybe_never_type: HashMap<Type /* TypeVar */, Type /* Type::Never */>,

    // We might fail to infer type of name bindings in patterns, because
    // we don't solve the types of patterns (will later be done by MatchFsm).
    pub pattern_name_bindings: HashSet<Span>,

    pub lang_items: HashMap<String, Span>,
    pub errors: Vec<TypeError>,
    pub warnings: Vec<TypeWarning>,
    pub intermediate_dir: String,
}

impl TypeSolver {
    pub fn new(lang_items: HashMap<String, Span>, intermediate_dir: String) -> Self {
        TypeSolver {
            type_vars: HashMap::new(),
            type_var_refs: HashMap::new(),
            maybe_never_type: HashMap::new(),
            pattern_name_bindings: HashSet::new(),
            lang_items,
            errors: vec![],
            warnings: vec![],
            intermediate_dir,
        }
    }

    pub fn apply_never_types(
        &mut self,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) {
        let mut never_types = vec![];

        for type_var in self.type_vars.keys() {
            match type_var {
                Type::Var { def_span, .. } => match types.get(def_span) {
                    None | Some(Type::Var { .. } | Type::GenericInstance { .. }) => {
                        if let Some(never_type) = self.maybe_never_type.get(type_var) {
                            never_types.push((type_var.clone(), never_type.clone()));
                        }
                    },
                    _ => {},
                },
                Type::GenericInstance { call, generic } => match generic_instances.get(&(*call, *generic)) {
                    None | Some(Type::Var { .. } | Type::GenericInstance { .. }) => {
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
                        match types.get_mut(def_span) {
                            Some(Type::Func { r#return, .. }) => {
                                *r#return = Box::new(never_type.clone());
                            },
                            _ => unreachable!(),
                        }
                    }

                    else {
                        types.insert(*def_span, never_type.clone());
                    }

                    self.substitute(type_var, never_type, types, generic_instances);
                },
                Type::GenericInstance { call, generic } => {
                    generic_instances.insert((*call, *generic), never_type.clone());
                    self.substitute(type_var, never_type, types, generic_instances);
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn check_all_types_infered(
        &mut self,
        types: &HashMap<Span, Type>,
        generic_instances: &HashMap<(Span, Span), Type>,
        generic_def_span_rev: &HashMap<Span, Span>,

        // If the compiler has enough information to dispatch a call, we treat that as successfully infered.
        dispatched_calls: &HashSet<(Span /* call */, Span /* generic */)>,
    ) -> Result<(), ()> {
        let mut has_error = false;

        for (type_var, id) in self.type_vars.iter() {
            match type_var {
                Type::Var { def_span, is_return } => match types.get(def_span) {
                    None | Some(Type::Var { .. } | Type::GenericInstance { .. }) => {
                        if self.pattern_name_bindings.contains(def_span) {
                            continue;
                        }

                        has_error = true;
                        self.errors.push(TypeError::CannotInferType {
                            id: *id,
                            span: *def_span,
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
                                    Type::Var { .. } | Type::GenericInstance { .. } => {
                                        has_error = true;
                                        self.errors.push(TypeError::CannotInferType {
                                            id: *id,
                                            span: *def_span,
                                            is_return: true,
                                        });
                                    },
                                    _ => {
                                        let type_vars = return_type.get_type_vars();

                                        if !type_vars.is_empty() {
                                            has_error = true;
                                            self.errors.push(TypeError::PartiallyInferedType {
                                                id: *id,
                                                span: *def_span,
                                                r#type: return_type,
                                                is_return: true,
                                            });
                                        }
                                    },
                                }
                            }

                            else {
                                has_error = true;
                                self.errors.push(TypeError::PartiallyInferedType {
                                    id: *id,
                                    span: *def_span,
                                    r#type: t.clone(),
                                    is_return: false,
                                });
                            }
                        }
                    },
                },
                Type::GenericInstance { call, generic } => {
                    if dispatched_calls.contains(&(*call, *generic)) {
                        continue;
                    }

                    match generic_instances.get(&(*call, *generic)) {
                        None | Some(Type::Var { .. } | Type::GenericInstance { .. }) => {
                            has_error = true;
                            self.errors.push(TypeError::CannotInferGenericType {
                                call: *call,
                                generic: *generic,
                                func_def: generic_def_span_rev.get(generic).map(|g| *g),
                            });
                        },
                        Some(t) => {
                            let type_vars = t.get_type_vars();

                            if !type_vars.is_empty() {
                                has_error = true;
                                self.errors.push(TypeError::PartiallyInferedGenericType {
                                    call: *call,
                                    generic: *generic,
                                    func_def: generic_def_span_rev.get(generic).map(|g| *g),
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

    /// It checks whether `lhs` is supertype of `rhs`. If so, it returns the supertype (`rhs`).
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

        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,

        // If it's checking a type argument (`Int` in `Option<Int>`), it doesn't
        // generate an error message, and its caller will.
        is_checking_argument: bool,

        // for helpful error messages
        lhs_span: Option<Span>,
        rhs_span: Option<Span>,
        context: ErrorContext,
        bidirectional: bool,
    ) -> Result<Type, ()> {
        match (lhs, rhs) {
            (Type::Static { def_span: exp_def, .. }, Type::Static { def_span: sub_def, .. }) => {
                if *exp_def == *sub_def {
                    Ok(lhs.clone())
                }

                else {
                    if !is_checking_argument {
                        self.errors.push(TypeError::UnexpectedType {
                            expected: lhs.clone(),
                            expected_span: lhs_span,
                            got: rhs.clone(),
                            got_span: rhs_span,
                            context: context.clone(),
                        });
                    }

                    Err(())
                }
            },
            (Type::Unit(_), Type::Unit(_)) => Ok(lhs.clone()),
            (Type::Never(_), Type::Never(_)) => Ok(lhs.clone()),
            (Type::Param { constructor: t1, args: args1, .. }, Type::Param { constructor: t2, args: args2, .. }) |
            (Type::Func { r#return: t1, params: args1, .. }, Type::Func { r#return: t2, params: args2, .. }) => {
                let t = match self.solve_supertype(
                    t1,
                    t2,
                    types,
                    generic_instances,
                    true,  // is_checking_argument
                    None,
                    None,
                    context.clone(),
                    bidirectional,
                ) {
                    Ok(t) => t,
                    Err(()) => {
                        if !is_checking_argument {
                            self.errors.push(TypeError::UnexpectedType {
                                expected: lhs.clone(),
                                expected_span: lhs_span,
                                got: rhs.clone(),
                                got_span: rhs_span,
                                context: context.clone(),
                            });
                        }

                        return Err(());
                    },
                };

                if args1.len() != args2.len() {
                    if !is_checking_argument {
                        self.errors.push(TypeError::UnexpectedType {
                            expected: lhs.clone(),
                            expected_span: lhs_span,
                            got: rhs.clone(),
                            got_span: rhs_span,
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
                        let (lhs_, rhs_, is_func) = if let Type::Func { .. } = lhs {
                            (&args2[i], &args1[i], true)
                        } else {
                            (&args1[i], &args2[i], false)
                        };

                        match self.solve_supertype(
                            lhs_,
                            rhs_,
                            types,
                            generic_instances,
                            true,  // is_checking_argument
                            None,
                            None,
                            ErrorContext::None,
                            bidirectional,
                        ) {
                            Ok(arg) => {
                                if is_func {
                                    args.push(arg);
                                }

                                else {
                                    args.push(rhs_.clone());
                                }
                            },
                            Err(()) => {
                                if !is_checking_argument {
                                    self.errors.push(TypeError::UnexpectedType {
                                        expected: lhs.clone(),
                                        expected_span: lhs_span,
                                        got: rhs.clone(),
                                        got_span: rhs_span,
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
                        match (lhs, rhs) {
                            (Type::Param { group_span, .. }, _) => Ok(Type::Param {
                                constructor: Box::new(t),
                                args,
                                group_span: *group_span,
                            }),
                            (Type::Func { fn_span, group_span, purity: p1, .. }, Type::Func { purity: p2, .. }) => {
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
                                                self.errors.push(TypeError::UnexpectedPurity {
                                                    expected_type: lhs.clone(),
                                                    expected_purity: *p1,
                                                    expected_span: lhs_span,
                                                    got_type: rhs.clone(),
                                                    got_purity: *p2,
                                                    got_span: rhs_span,
                                                });
                                            }

                                            return Err(());
                                        }
                                    },
                                };

                                Ok(Type::Func {
                                    fn_span: *fn_span,
                                    group_span: *group_span,
                                    params: args,
                                    r#return: Box::new(t),
                                    purity,
                                })
                            },
                            _ => unreachable!(),
                        }
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

                else if !*is_return1 && !*is_return2 {
                    match types.get(v1) {
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                        Some(type1) => {
                            let type1 = type1.clone();
                            return self.solve_supertype(
                                &type1,
                                t2,
                                types,
                                generic_instances,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                        None => {},
                    }

                    match types.get(v2) {
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                        Some(type2) => {
                            let type2 = type2.clone();
                            return self.solve_supertype(
                                t1,
                                &type2,
                                types,
                                generic_instances,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                        None => {},
                    }

                    types.insert(*v1, t2.clone());
                    self.add_type_var(t1.clone(), None);
                    self.add_type_var_ref(t1.clone(), t2.clone());
                    types.insert(*v2, t1.clone());
                    self.add_type_var(t2.clone(), None);
                    self.add_type_var_ref(t2.clone(), t1.clone());
                    Ok(t1.clone())
                }

                // `fn foo(x) = bar(x);`
                // in this case, we know that the return type of `foo` and the return type of `bar` are the same.
                else if *is_return1 && *is_return2 {
                    let (
                        Some(Type::Func { r#return: r1, .. }),
                        Some(Type::Func { r#return: r2, .. }),
                    ) = (types.get(v1), types.get(v2)) else { unreachable!() };

                    match (&**r1, &**r2) {
                        (
                            Type::Var { .. } | Type::GenericInstance { .. },
                            Type::Var { .. } | Type::GenericInstance { .. },
                        ) => {},
                        (r1, r2) => {
                            let r1 = r1.clone();
                            let r2 = r2.clone();
                            return self.solve_supertype(
                                &r1,
                                &r2,
                                types,
                                generic_instances,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                    }

                    types.insert(*v1, t2.clone());
                    self.add_type_var(t1.clone(), None);
                    self.add_type_var_ref(t1.clone(), t2.clone());
                    types.insert(*v2, t1.clone());
                    self.add_type_var(t2.clone(), None);
                    self.add_type_var_ref(t2.clone(), t1.clone());
                    Ok(t1.clone())
                }

                // `fn foo() = x;`, and we don't know the return type of `foo` and type of `x`.
                else {
                    panic!("TODO: {t1:?}, {t2:?}")
                }
            },
            (t1 @ Type::GenericInstance { call: c1, generic: g1 }, t2 @ Type::GenericInstance { call: c2, generic: g2 }) => {
                if *c1 == *c2 && *g1 == *g2 {
                    Ok(lhs.clone())
                }

                else {
                    match generic_instances.get(&(*c1, *g1)) {
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                        Some(type1) => {
                            let type1 = type1.clone();
                            return self.solve_supertype(
                                &type1,
                                t2,
                                types,
                                generic_instances,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                        None => {},
                    }

                    match generic_instances.get(&(*c2, *g2)) {
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                        Some(type2) => {
                            let type2 = type2.clone();
                            return self.solve_supertype(
                                t1,
                                &type2,
                                types,
                                generic_instances,
                                is_checking_argument,
                                lhs_span,
                                rhs_span,
                                ErrorContext::Deep,
                                bidirectional,
                            );
                        },
                        None => {},
                    }

                    generic_instances.insert((*c1, *g1), t2.clone());
                    self.add_type_var(t1.clone(), None);
                    self.add_type_var_ref(t1.clone(), t2.clone());
                    generic_instances.insert((*c2, *g2), t1.clone());
                    self.add_type_var(t2.clone(), None);
                    self.add_type_var_ref(t2.clone(), t1.clone());
                    Ok(t1.clone())
                }
            },
            (Type::GenericDef { .. }, _) | (_, Type::GenericDef { .. }) => {
                // We'll only type check/infer monomorphized functions.
                unreachable!()
            },
            (never @ Type::Never(_), concrete) | (concrete, never @ Type::Never(_)) => {
                let never_type_expected = matches!(lhs, Type::Never(_));

                // We don't solve the variable, because we might solve it with a more concrete type.
                // But we still have to remember that this variable might be `Type::Never`.
                // If we can't solve the variable, we'll assign `Type::Never` to the variable.
                match concrete {
                    Type::Var { .. } | Type::GenericInstance { .. } => {
                        self.maybe_never_type.insert(concrete.clone(), never.clone());
                    },
                    _ => {},
                }

                // `Type::Never` is subtype of every type, but `concrete` is not a
                // subtype of `Type::Never`.
                if bidirectional || !never_type_expected {
                    Ok(concrete.clone())
                } else {
                    self.errors.push(TypeError::UnexpectedType {
                        expected: lhs.clone(),
                        expected_span: lhs_span,
                        got: rhs.clone(),
                        got_span: rhs_span,
                        context: context.clone(),
                    });
                    Err(())
                }
            },
            (
                type_var @ Type::Var { def_span, is_return },
                concrete @ (Type::Static { .. } | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static { .. } | Type::Unit(_)),
                type_var @ Type::Var { def_span, is_return },
            ) => {
                let concrete_span = if let Type::Var { .. } = lhs {
                    rhs_span
                } else {
                    lhs_span
                };

                if *is_return {
                    // If previously infered type and newly infered type are different,
                    // that's an error!
                    match types.get(def_span) {
                        Some(Type::Func { r#return, .. }) => match &**r#return {
                            Type::Var { .. } | Type::GenericInstance { .. } => {},
                            prev_infered => {
                                let prev_infered = prev_infered.clone();

                                if let Err(()) = self.solve_supertype(
                                    &prev_infered,
                                    concrete,
                                    types,
                                    generic_instances,
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
                    match types.get_mut(def_span) {
                        Some(Type::Func { r#return, .. }) => {
                            *r#return = Box::new(concrete.clone());
                        },
                        _ => unreachable!(),
                    }
                }

                else {
                    // If previously infered type and newly infered type are different,
                    // that's an error!
                    match types.get(def_span) {
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                        Some(prev_infered) => {
                            let prev_infered = prev_infered.clone();

                            if let Err(()) = self.solve_supertype(
                                &prev_infered,
                                concrete,
                                types,
                                generic_instances,
                                false,
                                None,
                                concrete_span,
                                ErrorContext::InferedAgain { type_var: type_var.clone() },
                                bidirectional,
                            ) {
                                return Err(());
                            }
                        },
                        _ => {},
                    }

                    types.insert(*def_span, concrete.clone());
                }

                self.substitute(type_var, concrete, types, generic_instances);
                Ok(concrete.clone())
            },
            (
                type_var @ Type::Var { def_span, is_return },
                maybe_concrete @ (Type::Param { .. } | Type::Func { .. }),
            ) | (
                maybe_concrete @ (Type::Param { .. } | Type::Func { .. }),
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
                    match types.get(def_span) {
                        Some(Type::Func { r#return, .. }) => match &**r#return {
                            Type::Var { .. } | Type::GenericInstance { .. } => {},
                            prev_infered => {
                                let prev_infered = prev_infered.clone();

                                if let Err(()) = self.solve_supertype(
                                    &prev_infered,
                                    maybe_concrete,
                                    types,
                                    generic_instances,
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

                    match types.get_mut(def_span) {
                        Some(Type::Func { r#return, .. }) => {
                            *r#return = Box::new(maybe_concrete.clone());
                        },
                        _ => unreachable!(),
                    }
                }

                else {
                    // If previously infered type and newly infered type are different,
                    // that's an error!
                    match types.get(def_span) {
                        Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                        Some(prev_infered) => {
                            let prev_infered = prev_infered.clone();

                            if let Err(()) = self.solve_supertype(
                                &prev_infered,
                                maybe_concrete,
                                types,
                                generic_instances,
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

                    types.insert(*def_span, maybe_concrete.clone());
                }

                if ref_type_vars.is_empty() {
                    self.substitute(type_var, maybe_concrete, types, generic_instances);
                }

                else {
                    for ref_type_var in ref_type_vars.into_iter() {
                        self.add_type_var_ref(ref_type_var, type_var.clone());
                    }
                }

                Ok(maybe_concrete.clone())
            },
            (
                type_var @ Type::GenericInstance { call, generic },
                concrete @ (Type::Static { .. } | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static { .. } | Type::Unit(_)),
                type_var @ Type::GenericInstance { call, generic },
            ) => {
                let concrete_span = if let Type::Var { .. } = lhs {
                    rhs_span
                } else {
                    lhs_span
                };

                match generic_instances.get(&(*call, *generic)) {
                    Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                    Some(prev_infered) => {
                        let prev_infered = prev_infered.clone();

                        if let Err(()) = self.solve_supertype(
                            &prev_infered,
                            concrete,
                            types,
                            generic_instances,
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

                generic_instances.insert((*call, *generic), concrete.clone());
                self.substitute(type_var, concrete, types, generic_instances);
                Ok(concrete.clone())
            },
            (
                type_var @ Type::GenericInstance { call, generic },
                maybe_concrete @ (Type::Param { .. } | Type::Func { .. }),
            ) | (
                maybe_concrete @ (Type::Param { .. } | Type::Func { .. }),
                type_var @ Type::GenericInstance { call, generic },
            ) => {
                let ref_type_vars = maybe_concrete.get_type_vars();
                let concrete_span = if let Type::Var { .. } = lhs {
                    rhs_span
                } else {
                    lhs_span
                };

                match generic_instances.get(&(*call, *generic)) {
                    Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
                    Some(prev_infered) => {
                        let prev_infered = prev_infered.clone();

                        if let Err(()) = self.solve_supertype(
                            &prev_infered,
                            maybe_concrete,
                            types,
                            generic_instances,
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

                generic_instances.insert((*call, *generic), maybe_concrete.clone());

                if ref_type_vars.is_empty() {
                    self.substitute(type_var, maybe_concrete, types, generic_instances);
                }

                else {
                    for ref_type_var in ref_type_vars.into_iter() {
                        self.add_type_var_ref(ref_type_var, type_var.clone());
                    }
                }

                Ok(maybe_concrete.clone())
            },
            (
                Type::Static { .. } | Type::Unit(_) | Type::Param { .. } | Type::Func { .. },
                Type::Static { .. } | Type::Unit(_) | Type::Param { .. } | Type::Func { .. },
            ) => {
                if !is_checking_argument {
                    self.errors.push(TypeError::UnexpectedType {
                        expected: lhs.clone(),
                        expected_span: lhs_span,
                        got: rhs.clone(),
                        got_span: rhs_span,
                        context,
                    });
                }

                Err(())
            },
            (
                tv @ Type::Var { def_span, is_return },
                gi @ Type::GenericInstance { call, generic },
            ) | (
                gi @ Type::GenericInstance { call, generic },
                tv @ Type::Var { def_span, is_return },
            ) => {
                let (tv_span, gi_span) = if let Type::Var { .. } = lhs {
                    (lhs_span, rhs_span)
                } else {
                    (rhs_span, lhs_span)
                };

                match types.get(def_span) {
                    Some(Type::Var { .. } | Type::GenericInstance { .. }) => {},
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
                                    Type::Var { .. } | Type::GenericInstance { .. } => {},
                                    tv_concrete => {
                                        let tv_concrete = tv_concrete.clone();
                                        return self.solve_supertype(
                                            &tv_concrete,
                                            gi,
                                            types,
                                            generic_instances,
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
                                types,
                                generic_instances,
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

                // TODO: I want to `match generic_instances.get(&(*call, *generic))`, but it's
                //       complicated due to the `is_return` field...

                if !*is_return {
                    types.insert(*def_span, gi.clone());
                    self.add_type_var(tv.clone(), None);
                    self.add_type_var_ref(tv.clone(), gi.clone());
                    generic_instances.insert((*call, *generic), tv.clone());
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
    fn substitute(
        &mut self,
        type_var: &Type,
        r#type: &Type,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) {
        let ref_types = self.type_var_refs.get(&type_var).map(|refs| refs.to_vec()).unwrap_or(vec![]);
        let mut newly_completed_type_vars = vec![];

        for ref_type_var in ref_types.iter() {
            match ref_type_var {
                Type::Var { def_span, .. } => match types.get_mut(def_span) {
                    Some(ref_type) => {
                        ref_type.substitute(type_var, r#type);

                        if ref_type.get_type_vars().is_empty() {
                            newly_completed_type_vars.push(ref_type_var);
                        }
                    },
                    None => unreachable!(),
                },
                Type::GenericInstance { call, generic } => match generic_instances.get_mut(&(*call, *generic)) {
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
                Type::Var { def_span, .. } => {
                    let r#type = types.get(def_span).unwrap().clone();
                    self.substitute(type_var, &r#type, types, generic_instances);
                },
                Type::GenericInstance { call, generic } => {
                    let r#type = generic_instances.get_mut(&(*call, *generic)).unwrap().clone();
                    self.substitute(type_var, &r#type, types, generic_instances);
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items.get(lang_item) {
            Some(s) => *s,
            None => panic!("TODO: {lang_item:?}"),
        }
    }
}
