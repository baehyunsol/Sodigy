use crate::{Expr, Type};
use crate::error::{ErrorContext, TypeError, TypeErrorKind};
use crate::preludes::*;
use sodigy_mir::Callable;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::InfixOp;
use std::collections::HashMap;

pub struct Solver {
    pub preludes: Vec<InternedString>,
    pub infix_op_type_signatures: HashMap<InfixOp, Vec<Vec<Type>>>,
    pub errors: Vec<TypeError>,
}

impl Solver {
    pub fn new() -> Self {
        let preludes = get_preludes();

        // TODO: better way to manage this list?
        let infix_op_type_signatures = vec![
            (
                InfixOp::Add,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                    ],
                ],
            ),
        ].into_iter().collect();

        Solver {
            preludes,
            infix_op_type_signatures,
            errors: vec![],
        }
    }

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
    ) -> Result<Type, ()> {
        match expr {
            Expr::Identifier(id) => match types.get(&id.def_span) {
                Some(r#type) => Ok(r#type.clone()),
                None => Ok(Type::Var(id.def_span)),
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
                let cond_type = self.solve_expr(r#if.cond.as_ref(), types)?;

                match cond_type {
                    Type::Static(Span::Prelude(s)) if s == self.preludes[BOOL] => {},  // okay
                    _ => {
                        let _ = self.equal(
                            &Type::Static(Span::Prelude(self.preludes[BOOL])),
                            &cond_type,
                            types,
                            r#if.cond.error_span(),
                            None,
                            ErrorContext::IfConditionBool,
                        );
                    },
                }

                match (
                    self.solve_expr(r#if.true_value.as_ref(), types),
                    self.solve_expr(r#if.false_value.as_ref(), types),
                ) {
                    (Ok(true_type), Ok(false_type)) => {
                        self.equal(
                            &true_type,
                            &false_type,
                            types,
                            r#if.true_value.error_span(),
                            Some(r#if.false_value.error_span()),
                            ErrorContext::IfValueEqual,
                        )?;
                        Ok(true_type)
                    },
                    _ => Err(()),
                }
            },
            // The number of `args` is correct. Mir checked that.
            // ---- draft ----
            // 1. we can solve types of args, whether it's concrete or variable
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
            Expr::Call { func, args } => match func {
                Callable::GenericInfixOp { op, span } => {
                    let mut has_error = false;
                    let mut arg_types = Vec::with_capacity(args.len());

                    for arg in args.iter() {
                        match self.solve_expr(arg, types) {
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
                                kind: TypeErrorKind::InfixOpNotApplicable {
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
                            let mut has_error = false;

                            for i in 0..arg_types.len() {
                                if let Err(()) = self.equal(
                                    &candidate[i],
                                    &arg_types[i],
                                    types,
                                    args[i].error_span(),
                                    None,
                                    ErrorContext::None,  // TODO: do we need an error-context for this?
                                ) {
                                    has_error = true;
                                }
                            }

                            if has_error {
                                Err(())
                            }

                            else {
                                Ok(candidate.last().unwrap().clone())
                            }
                        },
                        2.. => todo!(),
                    }
                },
                _ => panic!("TODO: {func:?}"),
            },
            _ => todo!(),
        }
    }

    // It first checks whether `lhs` and `rhs` are equal. There's no subtyping in Sodigy.
    // If either operand is a type variable, it gets a new type equation.
    // It tries to solve the type equation. If it solves, it updates `types` and `self.`
    // If it finds non-sense while solving, it pushes the error to `self.errors` and returns `Err(())`.
    pub fn equal(
        &mut self,

        // It's a type equation `lhs == rhs`
        // If there's an error, the message would be "expected {lhs}, got {rhs}".
        lhs: &Type,
        rhs: &Type,

        types: &mut HashMap<Span, Type>,

        // for helpful error messages
        span: Span,
        extra_span: Option<Span>,
        context: ErrorContext,
    ) -> Result<(), ()> {
        match (lhs, rhs) {
            (Type::Unit(_), Type::Unit(_)) => Ok(()),
            (Type::Static(lhs_def), Type::Static(rhs_def)) => {
                if *lhs_def == *rhs_def {
                    Ok(())
                }

                else {
                    self.errors.push(TypeError {
                        kind: TypeErrorKind::UnexpectedType {
                            expected: lhs.clone(),
                            got: rhs.clone(),
                        },
                        span,
                        extra_span,
                        context,
                    });
                    Err(())
                }
            },
            (
                Type::Var(var),
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
                Type::Var(var),
            ) => {
                types.insert(*var, concrete.clone());
                self.substitute(*var, concrete, types)
            },
            (
                Type::GenericDef(_),
                concrete @ (Type::Static(_) | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static(_) | Type::Unit(_)),
                Type::GenericDef(_),
            ) => {
                self.errors.push(TypeError {
                    kind: TypeErrorKind::GenericIsNotGeneric {
                        got: concrete.clone(),
                    },
                    span,
                    extra_span,
                    context,
                });
                Err(())
            },
            _ => todo!(),
        }
    }

    fn substitute(&mut self, var: Span, r#type: &Type, types: &mut HashMap<Span, Type>) -> Result<(), ()> {
        // TODO: as of now, there's nothing to substitute (it doesn't store any type equations)
        Ok(())
    }

    fn get_possible_type_signatures(&self, op: InfixOp) -> &[Vec<Type>] {
        self.infix_op_type_signatures.get(&op).unwrap()
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
            (_, Type::Var(_) | Type::GenericInstance { .. }) => {},
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
