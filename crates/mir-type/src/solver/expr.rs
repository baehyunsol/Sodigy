use super::Solver;
use crate::{Expr, Type};
use crate::error::{ErrorContext, TypeError, TypeErrorKind};
use crate::preludes::*;
use sodigy_mir::Callable;
use sodigy_span::Span;
use sodigy_token::InfixOp;
use std::collections::HashMap;

impl Solver {
    // FIXME: there are A LOT OF heap allocations
    //
    // It can solve type of any expression, but the result maybe `Type::Var`.
    // If it finds new type equations while solving, it adds them to `type_equations`.
    //
    // It tries to find as many errors as possible before it returns.
    // Sometimes, it can solve the expr even though there's an error.
    // For example, `if 3 { 0 } else { 1 }` has an error, but its type
    // is definitely an integer. In this case, it pushes the error to the
    // solver and returns `Ok(Int)`.
    pub fn solve_expr(
        &mut self,
        expr: &Expr,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> Result<Type, ()> {
        match expr {
            Expr::Identifier(id) => match types.get(&id.def_span) {
                Some(r#type) => Ok(r#type.clone()),
                None => {
                    self.add_type_var(Type::Var { def_span: id.def_span, is_return: false }, Some(id.id));
                    Ok(Type::Var {
                        def_span: id.def_span,
                        is_return: false,
                    })
                },
            },
            Expr::Number { n, .. } => match n.is_integer {
                true => Ok(Type::Static(Span::Prelude(self.preludes[INT]))),
                false => Ok(Type::Static(Span::Prelude(self.preludes[NUMBER]))),
            },
            Expr::String { binary, .. } => match *binary {
                true => Ok(Type::Static(Span::Prelude(self.preludes[BYTES]))),
                false => Ok(Type::Static(Span::Prelude(self.preludes[STRING]))),
            },
            Expr::If(r#if) => {
                let cond_type = self.solve_expr(r#if.cond.as_ref(), types, generic_instances)?;

                match cond_type {
                    Type::Static(Span::Prelude(s)) if s == self.preludes[BOOL] => {},  // okay
                    _ => {
                        let _ = self.equal(
                            &Type::Static(Span::Prelude(self.preludes[BOOL])),
                            &cond_type,
                            types,
                            generic_instances,
                            r#if.cond.error_span(),
                            None,
                            ErrorContext::IfConditionBool,
                        );
                    },
                }

                match (
                    self.solve_expr(r#if.true_value.as_ref(), types, generic_instances),
                    self.solve_expr(r#if.false_value.as_ref(), types, generic_instances),
                ) {
                    (Ok(true_type), Ok(false_type)) => {
                        self.equal(
                            &true_type,
                            &false_type,
                            types,
                            generic_instances,
                            r#if.true_value.error_span(),
                            Some(r#if.false_value.error_span()),
                            ErrorContext::IfValueEqual,
                        )?;
                        Ok(true_type)
                    },
                    _ => Err(()),
                }
            },
            Expr::Block(block) => {
                let mut has_error = false;

                for r#let in block.lets.iter() {
                    if let Err(()) = self.solve_let(r#let, types, generic_instances) {
                        has_error = true;
                    }
                }

                for assert in block.asserts.iter() {
                    if let Err(()) = self.solve_assert(assert, types, generic_instances) {
                        has_error = true;
                    }
                }

                self.solve_expr(block.value.as_ref(), types, generic_instances)
            },
            Expr::FieldModifier { fields, lhs, rhs } => todo!(),
            // ---- draft ----
            // 1. we can solve types of args
            // 2. if callable is...
            //    - a function without generic
            //      - every arg must have a concrete type, so is the return type
            //      - it calls `equal` for all args, and returns the return type
            //    - a generic function
            //      - it first converts `Generic` to `GenericInstance` and does what
            //        a non-generic function does
            //    - an operator
            //      - it lists all the possible type signatures of the operator
            //        - todo: what if it's generic? I guess we have to use `GenericInstance` here
            //      - it finds applicable candidates in the list
            //      - if there are 0 match: type error
            //      - if there are exactly 1 match: we can solve this!
            //      - if there are multiple matches... we need another form of a type-variable.. :(
            Expr::Call { func, args, generic_defs, given_keyword_arguments } if generic_defs.is_empty() => {
                let mut has_error = false;
                let mut arg_types = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    match self.solve_expr(arg, types, generic_instances) {
                        Ok(arg_type) => {
                            arg_types.push(arg_type);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    return Err(());
                }

                match func {
                    Callable::Static { def_span, span } => match types.get(def_span) {
                        Some(Type::Func {
                            args: arg_defs,
                            r#return,
                            ..
                        }) => {
                            let arg_defs = arg_defs.clone();
                            let return_type: Type = *r#return.clone();
                            let span = *span;

                            // It doesn't check arg types if there are wrong number of args.
                            // Whether or not there're type errors with args, it returns the return type.
                            if arg_types.len() != arg_defs.len() {
                                self.errors.push(TypeError {
                                    kind: TypeErrorKind::WrongNumberOfArguments {
                                        expected: arg_defs,
                                        got: arg_types,
                                        given_keyword_arguments: given_keyword_arguments.to_vec(),
                                        arg_spans: args.iter().map(|arg| arg.error_span()).collect(),
                                    },
                                    span: func.error_span(),
                                    extra_span: None,
                                    context: ErrorContext::FuncArgs,
                                });
                            }

                            else {
                                for (i, arg_def) in arg_defs.iter().enumerate() {
                                    let _ = self.equal(
                                        arg_def,
                                        &arg_types[i],
                                        types,
                                        generic_instances,
                                        args[i].error_span(),
                                        Some(span),
                                        ErrorContext::FuncArgs,
                                    );
                                }
                            }

                            Ok(return_type)
                        },
                        Some(_) => todo!(),
                        None => todo!(),
                    },
                    Callable::ListInit { group_span } => {
                        // We can treat a list initialization (`[1, 2, 3]`) like calling a
                        // function with variadic arguments (`list.init(1, 2, 3)`).
                        // Here, `list.init` is a generic function `fn init<T>(args) -> [T]`.
                        // Then, an empty initialization is like calling a generic function
                        // but we don't know its generic yet.
                        if arg_types.is_empty() {
                            let type_var = Type::GenericInstance { call: *group_span, generic: Span::None };
                            self.add_type_var(type_var.clone(), None);

                            Ok(Type::Param {
                                r#type: Box::new(Type::Static(Span::Prelude(self.preludes[LIST]))),
                                args: vec![type_var],

                                // this is for the type annotation, hence None
                                group_span: Span::None,
                            })
                        }

                        else {
                            for i in 1..arg_types.len() {
                                let _ = self.equal(
                                    &arg_types[0],
                                    &arg_types[i],
                                    types,
                                    generic_instances,
                                    args[i].error_span(),
                                    Some(args[0].error_span()),
                                    ErrorContext::ListElementEqual,
                                );
                            }

                            Ok(Type::Param {
                                r#type: Box::new(Type::Static(Span::Prelude(self.preludes[LIST]))),
                                args: arg_types.drain(0..1).collect(),

                                // this is for the type annotation, hence None
                                group_span: Span::None,
                            })
                        }
                    },
                    Callable::Dynamic(func) => {
                        let func_type = self.solve_expr(func, types, generic_instances)?;

                        match func_type {
                            // TODO: What if there's a callable `Type::Static()` or `Type::Param {}`?
                            Type::Static(_) | Type::Unit(_) | Type::Param { .. } => {
                                self.errors.push(TypeError {
                                    kind: TypeErrorKind::NotCallable {
                                        r#type: func_type.clone(),
                                    },
                                    span: func.error_span(),
                                    extra_span: None,
                                    context: ErrorContext::None,
                                });
                                return Err(());
                            },

                            // Some generics are callable.
                            // I have to add a constraint.
                            Type::GenericDef(_) => todo!(),

                            Type::Func { args: arg_defs, r#return, .. } => {
                                // It doesn't check arg types if there are wrong number of args.
                                // Whether or not there're type errors with args, it returns the return type.
                                if arg_types.len() != arg_defs.len() {
                                    self.errors.push(TypeError {
                                        kind: TypeErrorKind::WrongNumberOfArguments {
                                            expected: arg_defs,
                                            got: arg_types,
                                            given_keyword_arguments: given_keyword_arguments.to_vec(),
                                            arg_spans: args.iter().map(|arg| arg.error_span()).collect(),
                                        },
                                        span: func.error_span(),
                                        extra_span: None,
                                        context: ErrorContext::None,
                                    });
                                }

                                else {
                                    for i in 0..arg_defs.len() {
                                        let _ = self.equal(
                                            &arg_defs[i],
                                            &arg_types[i],
                                            types,
                                            generic_instances,
                                            args[i].error_span(),
                                            None,
                                            ErrorContext::FuncArgs,
                                        );
                                    }
                                }

                                Ok(*r#return.clone())
                            },
                            _ => todo!(),
                        }
                    },
                    Callable::GenericInfixOp { op: InfixOp::Eq, span } => {
                        let _ = self.equal(
                            &arg_types[0],
                            &arg_types[1],
                            types,
                            generic_instances,
                            args[1].error_span(),
                            Some(args[0].error_span()),
                            ErrorContext::EqValueEqual,
                        );

                        Ok(Type::Static(Span::Prelude(self.preludes[BOOL])))
                    },
                    Callable::GenericInfixOp { op, span } => {
                        let type_signatures = self.get_possible_type_signatures(*op);
                        let mut candidates = vec![];

                        for type_signature in type_signatures.iter() {
                            if applicable(
                                type_signature,
                                &arg_types,
                            ) {
                                candidates.push(type_signature);
                            }
                        }

                        // Let's say `op` is `Op::Add`.
                        // Then the type signatures would be `[[Int, Int, Int], [Number, Number, Number], ... (and maybe more) ...]`.
                        // `candidates` filters out type signatures that are not compatible with `arg_types`.
                        match candidates.len() {
                            0 => {
                                self.errors.push(TypeError {
                                    kind: TypeErrorKind::CannotApplyInfixOp {
                                        op: *op,
                                        arg_types,
                                    },
                                    span: *span,
                                    extra_span: None,
                                    context: ErrorContext::None,
                                });
                                Err(())
                            },
                            1 => {
                                let candidate = candidates[0].clone();

                                for i in 0..arg_types.len() {
                                    let _ = self.equal(
                                        &candidate[i],
                                        &arg_types[i],
                                        types,
                                        generic_instances,
                                        args[i].error_span(),
                                        None,
                                        ErrorContext::None,  // TODO: do we need an error-context for this?
                                    );
                                }

                                Ok(candidate.last().unwrap().clone())
                            },
                            2.. => todo!(),
                        }
                    },
                    _ => panic!("TODO: {func:?}"),
                }
            },
            _ => panic!("TODO: {expr:?}"),
        }
    }
}

// `type_signature.len() == arg_types.len() + 1` because the last element of
// `type_signature` is the return type.
fn applicable(
    type_signature: &[Type],
    arg_types: &[Type],
) -> bool {
    assert_eq!(type_signature.len(), arg_types.len() + 1);

    for i in 0..arg_types.len() {
        // TODO: there must be an error in this match statement.
        match (
            &type_signature[i],
            &arg_types[i],
        ) {
            (_, Type::Var { .. } | Type::GenericInstance { .. }) => {},
            (Type::Static(s1), Type::Static(s2)) if *s1 == *s2 => {},
            (Type::Unit(_), Type::Unit(_)) => {},
            (Type::Param { .. }, _) |
            (_, Type::Param { .. }) => todo!(),
            _ => {
                return false;
            },
        }
    }

    true
}
