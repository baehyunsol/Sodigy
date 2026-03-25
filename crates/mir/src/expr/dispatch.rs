use super::Expr;
use crate::{Callable, Type};
use sodigy_hir::FuncShape;
use sodigy_parse::Field;
use sodigy_span::Span;
use std::collections::HashMap;

impl Expr {
    pub fn dispatch(
        &mut self,
        generics: &HashMap<Span, Span>,
        associated_funcs: &HashMap<Span, Span>,
        func_shapes: &HashMap<Span, FuncShape>,
        generic_args: &mut HashMap<(Span, Span), Type>,
    ) {
        match self {
            // TODO: I guess we have to dispatch identifiers, too?
            //       e.g. let's say `add` is a generic function
            //       `let x: [Fn(Int, Int) -> Int] = [add, sub, mul, div];`
            //       Then we have to dispatch the identifiers in the list.
            Expr::Ident(_) => {},
            Expr::Constant(_) => {},
            Expr::If(r#if) => {
                r#if.cond.dispatch(generics, associated_funcs, func_shapes, generic_args);
                r#if.true_value.dispatch(generics, associated_funcs, func_shapes, generic_args);
                r#if.false_value.dispatch(generics, associated_funcs, func_shapes, generic_args);
            },
            Expr::Match(r#match) => {
                r#match.scrutinee.dispatch(generics, associated_funcs, func_shapes, generic_args);

                for arm in r#match.arms.iter_mut() {
                    if let Some(guard) = &mut arm.guard {
                        guard.dispatch(generics, associated_funcs, func_shapes, generic_args);
                    }

                    arm.value.dispatch(generics, associated_funcs, func_shapes, generic_args);
                }
            },
            Expr::Block(block) => {
                for r#let in block.lets.iter_mut() {
                    r#let.value.dispatch(generics, associated_funcs, func_shapes, generic_args);
                }

                for assert in block.asserts.iter_mut() {
                    assert.value.dispatch(generics, associated_funcs, func_shapes, generic_args);

                    if let Some(note) = &mut assert.note {
                        note.dispatch(generics, associated_funcs, func_shapes, generic_args);
                    }
                }

                block.value.dispatch(generics, associated_funcs, func_shapes, generic_args);
            },
            Expr::Field { lhs, fields } => {
                lhs.dispatch(generics, associated_funcs, func_shapes, generic_args);

                // `x.y.push` -> `\(z) => associated_func::push(x.y, z)`
                if let Some(Field::Name { name_span, .. }) = fields.last() && let Some(poly_def_span) = associated_funcs.get(name_span) {
                    // We can't do this because closure is not implemented yet
                    todo!()
                }
            },
            Expr::FieldUpdate { lhs, rhs, .. } => {
                lhs.dispatch(generics, associated_funcs, func_shapes, generic_args);
                rhs.dispatch(generics, associated_funcs, func_shapes, generic_args);
            },
            Expr::Call { func, args, arg_group_span, types, given_keyword_args } => {
                let dispatch = match func {
                    Callable::Static { span, .. } => match generics.get(span) {
                        Some(new_def_span) => Some((new_def_span.clone(), span.clone())),
                        None => None,
                    },
                    Callable::Dynamic(f) => {
                        if let Expr::Field { lhs, fields } = &**f {
                            // `x.y.push(z)` -> `associated_func::push(x.y, z)`
                            if let Some(Field::Name { name_span, .. }) = fields.last() && let Some(poly_def_span) = associated_funcs.get(name_span) {
                                let new_lhs = if fields.len() == 1 {
                                    lhs.as_ref().clone()
                                } else {
                                    Expr::Field {
                                        lhs: lhs.clone(),
                                        fields: fields[..(fields.len() - 1)].to_vec(),
                                    }
                                };
                                let mut new_args = vec![new_lhs];
                                new_args.extend(args.to_vec());

                                for arg in args.iter_mut() {
                                    arg.dispatch(generics, associated_funcs, func_shapes, generic_args);
                                }

                                *self = Expr::Call {
                                    func: Callable::Static {
                                        def_span: poly_def_span.clone(),
                                        span: name_span.clone(),
                                    },
                                    args: new_args,
                                    arg_group_span: arg_group_span.clone(),
                                    types: types.clone(),
                                    given_keyword_args: given_keyword_args.clone(),
                                };
                                return;
                            }
                        }

                        f.dispatch(generics, associated_funcs, func_shapes, generic_args);
                        None
                    },
                    _ => None,
                };

                if let Some((def_span, span)) = &dispatch {
                    *func = Callable::Static { def_span: def_span.clone(), span: span.clone() };
                    *types = None;

                    match func_shapes.get(def_span) {
                        Some(func_shape) => {
                            for generic in func_shape.generics.iter() {
                                generic_args.insert(
                                    (span.clone(), generic.name_span.clone()),
                                    Type::GenericArg {
                                        call: span.clone(),
                                        generic: generic.name_span.clone(),
                                    },
                                );
                            }
                        },
                        None => unreachable!(),
                    }
                }

                for arg in args.iter_mut() {
                    arg.dispatch(generics, associated_funcs, func_shapes, generic_args);
                }
            },
        }
    }
}
