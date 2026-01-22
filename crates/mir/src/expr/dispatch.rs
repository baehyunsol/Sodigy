use super::Expr;
use crate::{Callable, Type};
use sodigy_hir::FuncShape;
use sodigy_span::Span;
use std::collections::HashMap;

impl Expr {
    pub fn dispatch(
        &mut self,
        map: &HashMap<Span, Span>,
        func_shapes: &HashMap<Span, FuncShape>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) {
        match self {
            // TODO: I guess we have to dispatch identifiers, too?
            //       e.g. let's say `add` is a generic function
            //       `let x: [Fn(Int, Int) -> Int] = [add, sub, mul, div];`
            //       Then we have to dispatch the identifiers in the list.
            Expr::Ident(_) => {},
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => {},
            Expr::If(r#if) => {
                r#if.cond.dispatch(map, func_shapes, generic_instances);
                r#if.true_value.dispatch(map, func_shapes, generic_instances);
                r#if.false_value.dispatch(map, func_shapes, generic_instances);
            },
            Expr::Match(r#match) => {
                r#match.scrutinee.dispatch(map, func_shapes, generic_instances);

                for arm in r#match.arms.iter_mut() {
                    if let Some(guard) = &mut arm.guard {
                        guard.dispatch(map, func_shapes, generic_instances);
                    }

                    arm.value.dispatch(map, func_shapes, generic_instances);
                }
            },
            Expr::Block(block) => {
                for r#let in block.lets.iter_mut() {
                    r#let.value.dispatch(map, func_shapes, generic_instances);
                }

                for assert in block.asserts.iter_mut() {
                    assert.value.dispatch(map, func_shapes, generic_instances);

                    if let Some(note) = &mut assert.note {
                        note.dispatch(map, func_shapes, generic_instances);
                    }
                }

                block.value.dispatch(map, func_shapes, generic_instances);
            },
            Expr::Path { lhs, .. } => {
                lhs.dispatch(map, func_shapes, generic_instances);
            },
            Expr::FieldModifier { lhs, rhs, .. } => {
                lhs.dispatch(map, func_shapes, generic_instances);
                rhs.dispatch(map, func_shapes, generic_instances);
            },
            Expr::Call { func, args, generic_defs, .. } => {
                let dispatch = match func {
                    Callable::Static { span, .. } => match map.get(span) {
                        Some(new_def_span) => Some((*new_def_span, *span)),
                        None => None,
                    },
                    _ => None,
                };

                if let Some((def_span, span)) = dispatch {
                    *func = Callable::Static { def_span, span };

                    let mut new_generic_defs = vec![];

                    match func_shapes.get(&def_span) {
                        Some(func_shape) => {
                            for generic_def in func_shape.generics.iter() {
                                generic_instances.insert(
                                    (span, generic_def.name_span),
                                    Type::GenericInstance {
                                        call: span,
                                        generic: generic_def.name_span,
                                    },
                                );
                                new_generic_defs.push(generic_def.name_span);
                            }
                        },
                        None => unreachable!(),
                    }

                    *generic_defs = new_generic_defs;
                }

                for arg in args.iter_mut() {
                    arg.dispatch(map, func_shapes, generic_instances);
                }
            },
        }
    }
}
