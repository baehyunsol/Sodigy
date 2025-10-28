use crate::Type;
use crate::error::{ErrorContext, TypeError};
use crate::preludes::*;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{InfixOp, PostfixOp, PrefixOp};
use std::collections::hash_map::{Entry, HashMap};

mod assert;
mod expr;
mod func;
mod r#let;

// When a type-variable is solved, it removes an entry in `type_var_refs`, but
// not in `type_vars`, because
// 1. We'll later use `type_vars` to distinguish what're infered types and what're annotated types.
// 2. If we don't remove entries in `type_var_refs`, cyclic type vars will cause a stack overflow.
pub struct Solver {
    // Whenever `types.get(span)` returns `None`, it creates a type variable
    // and inserts the `span` to this hash set. It's later used to check
    // if all the type variables are infered.
    // If the type variable is from a type annotation and a name is bound to
    // the type annotation, it also collects the name: that'd be helpful when
    // creating error messages.
    //
    // TODO: it has to collect `Type::GenericInstance`.
    pub type_vars: HashMap<Type, Option<InternedString>>,

    // If a type variable references another type variable, we have to track the relation.
    // For example, if a type of function `add` is `TypeVar(add) = Fn(TypeVar(x), TypeVar(y)) -> Int`,
    // we have to update `TypeVar(add)` when `TypeVar(x)` is updated. So, we `type_var_refs.get(x)`
    // will give you a vector with `add`.
    // If a type variable references itself, that should not be included in the Vec<Span>.
    pub type_var_refs: HashMap<Type, Vec<Type>>,

    // When it solves a generic function, say `foo<T>`, it treats `T` like a concrete type.
    // It first solves unknown types in the function, and collects constraints of `T`.
    // For example, if it sees a type equation `T = Int`, it inserts `Int` to the constraints of `T`.
    // Later, it checks if instances of `foo` all satisfies the constraints.
    pub generic_constraints: HashMap<Span, Vec<Type>>,

    pub preludes: Vec<InternedString>,
    pub prefix_op_type_signatures: HashMap<PrefixOp, Vec<Vec<Type>>>,
    pub infix_op_type_signatures: HashMap<InfixOp, Vec<Vec<Type>>>,
    pub postfix_op_type_signatures: HashMap<PostfixOp, Vec<Vec<Type>>>,
    pub errors: Vec<TypeError>,
}

impl Solver {
    pub fn new() -> Self {
        let preludes = get_preludes();

        // TODO: better way to manage this list?
        // TODO: how does it represent an index operator? it's generic...
        let prefix_op_type_signatures = vec![
            (
                PrefixOp::Not,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[BOOL])),
                        Type::Static(Span::Prelude(preludes[BOOL])),
                    ],
                ],
            ), (
                PrefixOp::Neg,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                    ],
                    vec![
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                    ],
                ],
            ),
        ].into_iter().collect();
        let infix_op_type_signatures = vec![
            (
                InfixOp::Add,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                    ],
                    vec![
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                    ],
                ],
            ), (
                InfixOp::Mul,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                    ],
                    vec![
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                    ],
                ],
            ), (
                InfixOp::Gt,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[BOOL])),
                    ],
                    vec![
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[BOOL])),
                    ],
                ],
            ), (
                InfixOp::Lt,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[BOOL])),
                    ],
                    vec![
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[NUMBER])),
                        Type::Static(Span::Prelude(preludes[BOOL])),
                    ],
                ],
            ),
        ].into_iter().collect();
        let postfix_op_type_signatures = vec![
            (
                PostfixOp::Range { inclusive: true },
                vec![
                    // TODO: Range type
                ],
            ), (
                PostfixOp::Range { inclusive: false },
                vec![
                    // TODO: Range type
                ],
            ), (
                PostfixOp::QuestionMark,
                vec![
                    // TODO: spec for this operator
                ],
            ),
        ].into_iter().collect();

        Solver {
            type_vars: HashMap::new(),
            type_var_refs: HashMap::new(),
            generic_constraints: HashMap::new(),
            preludes,
            prefix_op_type_signatures,
            infix_op_type_signatures,
            postfix_op_type_signatures,
            errors: vec![],
        }
    }

    pub fn check_all_types_infered(
        &mut self,
        types: &HashMap<Span, Type>,
        generic_instances: &HashMap<(Span, Span), Type>,
        generic_def_span_rev: &HashMap<Span, Span>,
    ) {
        for (type_var, id) in self.type_vars.iter() {
            match type_var {
                Type::Var { def_span, .. } => match types.get(def_span) {
                    None | Some(Type::Var { .. } | Type::GenericInstance { .. }) => {
                        self.errors.push(TypeError::CannotInferType {
                            id: *id,
                            span: *def_span,
                        });
                    },
                    Some(t) => {
                        let type_vars = t.get_type_vars();
    
                        if !type_vars.is_empty() {
                            self.errors.push(TypeError::PartiallyInferedType { id: *id, span: *def_span, r#type: t.clone() });
                        }
                    },
                },
                Type::GenericInstance { call, generic } => match generic_instances.get(&(*call, *generic)) {
                    None | Some(Type::Var { .. } | Type::GenericInstance { .. }) => {
                        self.errors.push(TypeError::CannotInferGenericType {
                            call: *call,
                            generic: *generic,
                            func_def: generic_def_span_rev.get(generic).map(|g| *g),
                        });
                    },
                    Some(t) => {
                        let type_vars = t.get_type_vars();

                        if !type_vars.is_empty() {
                            self.errors.push(TypeError::PartiallyInferedGenericType {
                                call: *call,
                                generic: *generic,
                                func_def: generic_def_span_rev.get(generic).map(|g| *g),
                                r#type: t.clone(),
                            });
                        }
                    },
                },
                _ => unreachable!(),
            }
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

    pub fn add_generic_constraint(&mut self, generic_def_span: Span, r#type: &Type) {
        let r#type = r#type.clone();

        match self.generic_constraints.entry(generic_def_span) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(r#type);
            },
            Entry::Vacant(e) => {
                e.insert(vec![r#type]);
            },
        }
    }

    // It first checks whether `lhs` and `rhs` are equal. There's no subtyping in Sodigy. (TODO: there is)
    // If either operand is a type variable, it creates a new type equation.
    // It tries to solve the type equation. If it solves, it updates `types` and `self.`
    // If it finds non-sense while solving, it pushes the error to `self.errors` and returns `Err(())`.
    pub fn equal(
        &mut self,

        // It's a type equation `lhs == rhs`
        // If there's an error, the message would be "expected {lhs}, got {rhs}".
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
    ) -> Result<(), ()> {
        match (lhs, rhs) {
            (Type::Unit(_), Type::Unit(_)) => Ok(()),
            (Type::Static(lhs_def), Type::Static(rhs_def)) => {
                if *lhs_def == *rhs_def {
                    Ok(())
                }

                else {
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
                }
            },
            (Type::Param { r#type: t1, args: args1, .. }, Type::Param { r#type: t2, args: args2, .. }) |
            (Type::Func { r#return: t1, args: args1, .. }, Type::Func { r#return: t2, args: args2, .. }) => {
                if let Err(()) = self.equal(
                    t1,
                    t2,
                    types,
                    generic_instances,
                    true,  // is_checking_argument
                    None,
                    None,
                    context,
                ) {
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
                }

                else if args1.len() != args2.len() {
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
                }

                else {
                    let mut has_error = false;

                    for i in 0..args1.len() {
                        if let Err(()) = self.equal(
                            &args1[i],
                            &args2[i],
                            types,
                            generic_instances,
                            true,  // is_checking_argument
                            None,
                            None,
                            ErrorContext::None,
                        ) {
                            if !is_checking_argument {
                                self.errors.push(TypeError::UnexpectedType {
                                    expected: lhs.clone(),
                                    expected_span: lhs_span,
                                    got: rhs.clone(),
                                    got_span: rhs_span,
                                    context,
                                });
                            }
                            has_error = true;
                        }
                    }

                    if has_error {
                        Err(())
                    }

                    else {
                        Ok(())
                    }
                }
            },
            (
                type_var @ Type::Var { def_span, is_return },
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
                type_var @ Type::Var { def_span, is_return },
            ) => {
                if *is_return {
                    match types.get_mut(def_span) {
                        Some(Type::Func { r#return, .. }) => {
                            *r#return = Box::new(concrete.clone());
                        },
                        _ => unreachable!(),
                    }
                }

                else {
                    types.insert(*def_span, concrete.clone());
                }

                self.substitute(type_var, concrete, types, generic_instances);
                Ok(())
            },
            (
                type_var @ Type::Var { def_span, is_return },
                maybe_concrete @ (Type::Func { .. } | Type::Param { .. }),
            ) | (
                maybe_concrete @ (Type::Func { .. } | Type::Param { .. }),
                type_var @ Type::Var { def_span, is_return },
            ) => {
                let ref_type_vars = maybe_concrete.get_type_vars();

                if ref_type_vars.is_empty() {
                    if *is_return {
                        match types.get_mut(def_span) {
                            Some(Type::Func { r#return, .. }) => {
                                *r#return = Box::new(maybe_concrete.clone());
                            },
                            _ => unreachable!(),
                        }
                    }

                    else {
                        types.insert(*def_span, maybe_concrete.clone());
                    }

                    self.substitute(type_var, maybe_concrete, types, generic_instances);
                }

                else {
                    for ref_type_var in ref_type_vars.into_iter() {
                        self.add_type_var_ref(ref_type_var, type_var.clone());
                    }
                }

                Ok(())
            },
            (Type::Var { def_span: v1, .. }, Type::Var { def_span: v2, .. }) if *v1 == *v2 => {
                // nop
                Ok(())
            },
            (Type::GenericInstance { call: c1, generic: g1 }, Type::GenericInstance { call: c2, generic: g2 }) if *c1 == *c2 && *g1 == *g2 => {
                // nop
                Ok(())
            },
            (
                t1 @ Type::Var { def_span: v1, is_return: false },
                t2 @ Type::Var { def_span: v2, is_return: false },
            ) => {
                if let Some(type1) = types.get(v1) {
                    let type1 = type1.clone();
                    self.equal(
                        &type1,
                        rhs,
                        types,
                        generic_instances,
                        is_checking_argument,
                        lhs_span,
                        rhs_span,
                        ErrorContext::Deep,
                    )
                }

                else if let Some(type2) = types.get(v2) {
                    let type2 = type2.clone();
                    self.equal(
                        lhs,
                        &type2,
                        types,
                        generic_instances,
                        is_checking_argument,
                        lhs_span,
                        rhs_span,
                        ErrorContext::Deep,
                    )
                }

                else {
                    types.insert(*v1, t2.clone());
                    self.add_type_var(t1.clone(), None);
                    self.add_type_var_ref(t1.clone(), t2.clone());
                    types.insert(*v2, t1.clone());
                    self.add_type_var(t2.clone(), None);
                    self.add_type_var_ref(t2.clone(), t1.clone());
                    Ok(())
                }
            },
            (t1 @ Type::GenericDef(g1), t2 @ Type::GenericDef(g2)) => {
                self.add_generic_constraint(*g1, t2);
                self.add_generic_constraint(*g2, t1);
                Ok(())
            },
            (Type::GenericDef(generic), constraint) |
            (constraint, Type::GenericDef(generic)) => {
                self.add_generic_constraint(*generic, constraint);
                Ok(())
            },
            _ => panic!("TODO: {:?}", (lhs, rhs)),
        }
    }

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
                Type::GenericInstance { call, generic } => todo!(),
                _ => unreachable!(),
            }
        }
    }

    fn get_prefix_op_type_signatures(&self, op: PrefixOp) -> &[Vec<Type>] {
        match self.prefix_op_type_signatures.get(&op) {
            Some(tys) => tys,
            None => panic!("TODO: type signatures for {op:?}"),
        }
    }

    fn get_infix_op_type_signatures(&self, op: InfixOp) -> &[Vec<Type>] {
        match self.infix_op_type_signatures.get(&op) {
            Some(tys) => tys,
            None => panic!("TODO: type signatures for {op:?}"),
        }
    }

    fn get_postfix_op_type_signatures(&self, op: PostfixOp) -> &[Vec<Type>] {
        match self.postfix_op_type_signatures.get(&op) {
            Some(tys) => tys,
            None => panic!("TODO: type signatures for {op:?}"),
        }
    }
}
