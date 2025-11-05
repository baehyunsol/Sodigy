use super::Solver;
use crate::{Expr, Type};
use crate::error::{ErrorContext, TypeError};
use sodigy_mir::Callable;
use sodigy_span::Span;
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
                true => Ok(Type::Static(self.get_lang_item_span("type.Int"))),
                false => Ok(Type::Static(self.get_lang_item_span("type.Number"))),
            },
            Expr::String { binary, .. } => match *binary {
                true => Ok(Type::Static(self.get_lang_item_span("type.Bytes"))),
                false => Ok(Type::Static(self.get_lang_item_span("type.String"))),
            },
            Expr::If(r#if) => {
                let cond_type = self.solve_expr(r#if.cond.as_ref(), types, generic_instances)?;

                match cond_type {
                    Type::Static(s) if s == self.get_lang_item_span("type.Bool") => {},  // okay
                    _ => {
                        let _ = self.equal(
                            &Type::Static(self.get_lang_item_span("type.Bool")),
                            &cond_type,
                            types,
                            generic_instances,
                            false,
                            None,
                            Some(r#if.cond.error_span()),
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
                            false,
                            Some(r#if.true_value.error_span()),
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
                                    let _ = self.equal(
                                        arg_def,
                                        &arg_types[i],
                                        types,
                                        generic_instances,
                                        false,
                                        None,
                                        Some(args[i].error_span()),
                                        ErrorContext::FuncArgs,
                                    );
                                }
                            }

                            Ok(return_type)
                        },
                        Some(_) => todo!(),
                        None => todo!(),
                    },
                    Callable::TupleInit { group_span } => Ok(Type::Param {
                        // `Type::Unit`'s `group_span` is of type annotation,
                        // and `Callable::TupleInit`'s `group_span` is of the expression/
                        r#type: Box::new(Type::Unit(Span::None)),
                        args: arg_types,

                        // this is for the type annotation, hence None
                        group_span: Span::None,
                    }),
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
                                r#type: Box::new(Type::Static(self.get_lang_item_span("type.List"))),
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
                                    false,
                                    Some(args[0].error_span()),
                                    Some(args[i].error_span()),
                                    ErrorContext::ListElementEqual,
                                );
                            }

                            Ok(Type::Param {
                                r#type: Box::new(Type::Static(self.get_lang_item_span("type.List"))),
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
                                self.errors.push(TypeError::NotCallable {
                                    r#type: func_type.clone(),
                                    func_span: func.error_span(),
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
                                        let _ = self.equal(
                                            &arg_defs[i],
                                            &arg_types[i],
                                            types,
                                            generic_instances,
                                            false,
                                            None,
                                            Some(args[i].error_span()),
                                            ErrorContext::FuncArgs,
                                        );
                                    }
                                }

                                Ok(*r#return.clone())
                            },
                            _ => todo!(),
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
