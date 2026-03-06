use super::Monomorphization;
use crate::Session;
use sodigy_mir::{Callable, Expr};
use sodigy_name_analysis::NameOrigin;

impl Session {
    pub fn monomorphize_expr(&mut self, expr: &mut Expr, monomorphization: &Monomorphization) {
        match expr {
            Expr::Ident(id) => {
                id.span = id.span.monomorphize(monomorphization.id);

                match id.origin {
                    NameOrigin::FuncParam { .. } | NameOrigin::Local { .. } => {
                        id.def_span = id.def_span.monomorphize(monomorphization.id);
                    },
                    _ => {},
                }
            },
            Expr::Constant(c) => {
                *c = c.monomorphize(monomorphization.id);
            },
            Expr::If(r#if) => {
                r#if.if_span = r#if.if_span.monomorphize(monomorphization.id);
                r#if.else_span = r#if.else_span.monomorphize(monomorphization.id);
                r#if.true_group_span = r#if.true_group_span.monomorphize(monomorphization.id);
                r#if.false_group_span = r#if.false_group_span.monomorphize(monomorphization.id);
                self.monomorphize_expr(&mut r#if.cond, monomorphization);
                self.monomorphize_expr(&mut r#if.true_value, monomorphization);
                self.monomorphize_expr(&mut r#if.false_value, monomorphization);
            },
            Expr::Match(r#match) => {
                r#match.keyword_span = r#match.keyword_span.monomorphize(monomorphization.id);
                r#match.group_span = r#match.group_span.monomorphize(monomorphization.id);
                self.monomorphize_expr(&mut r#match.scrutinee, monomorphization);

                for arm in r#match.arms.iter_mut() {
                    self.monomorphize_pattern(&mut arm.pattern, monomorphization);
                    self.monomorphize_expr(&mut arm.value, monomorphization);

                    if let Some(guard) = &mut arm.guard {
                        self.monomorphize_expr(guard, monomorphization);
                    }
                }
            },
            Expr::Block(block) => {
                block.group_span = block.group_span.monomorphize(monomorphization.id);
                self.monomorphize_expr(&mut block.value, monomorphization);

                for r#let in block.lets.iter_mut() {
                    let new_type = match self.types.get(&r#let.name_span) {
                        Some(r#type) => {
                            let mut r#type = r#type.clone();

                            for (generic_param, generic_arg) in monomorphization.generics.iter() {
                                r#type.substitute_generic_param(generic_param, generic_arg);
                            }

                            r#type
                        },
                        None => unreachable!(),
                    };

                    r#let.keyword_span = r#let.keyword_span.monomorphize(monomorphization.id);
                    r#let.name_span = r#let.name_span.monomorphize(monomorphization.id);
                    r#let.type_annot_span = r#let.type_annot_span.map(|span| span.monomorphize(monomorphization.id));
                    self.monomorphize_expr(&mut r#let.value, monomorphization);
                    // TODO: do we have to change `LetOrigin`?

                    self.types.insert(r#let.name_span, new_type);
                }

                for assert in block.asserts.iter_mut() {
                    assert.keyword_span = assert.keyword_span.monomorphize(monomorphization.id);
                    self.monomorphize_expr(&mut assert.value, monomorphization);

                    if let Some(note) = &mut assert.note {
                        self.monomorphize_expr(note, monomorphization);
                    }

                    if let Some(note_decorator_span) = &mut assert.note_decorator_span {
                        *note_decorator_span = note_decorator_span.monomorphize(monomorphization.id);
                    }
                }
            },
            Expr::Field { lhs, .. } => {
                self.monomorphize_expr(lhs, monomorphization);
            },
            Expr::FieldUpdate { .. } => todo!(),
            Expr::Call { func, args, arg_group_span, .. } => {
                match func {
                    Callable::Static { span, .. } |
                    Callable::StructInit { span, .. } |
                    Callable::TupleInit { group_span: span, .. } |
                    Callable::ListInit { group_span: span, .. } => {
                        *span = span.monomorphize(monomorphization.id);
                    },
                    Callable::Dynamic(c) => todo!(),
                }

                for arg in args.iter_mut() {
                    self.monomorphize_expr(arg, monomorphization);
                }

                *arg_group_span = arg_group_span.monomorphize(monomorphization.id);
            },
        }
    }
}
