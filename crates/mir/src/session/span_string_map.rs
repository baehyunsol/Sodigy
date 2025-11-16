use super::Session;
use crate::{Assert, Callable, Expr, Func, Let};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Session {
    pub fn init_span_string_map(&mut self) {
        if self.span_string_map.is_some() {
            return;
        }

        let mut result = HashMap::new();

        for r#let in self.lets.iter() {
            self.init_span_string_map_let(r#let, &mut result);
        }

        for func in self.funcs.iter() {
            self.init_span_string_map_func(func, &mut result);
        }

        for assert in self.asserts.iter() {
            self.init_span_string_map_assert(assert, &mut result);
        }

        self.span_string_map = Some(result);
    }

    pub fn init_span_string_map_let(&self, r#let: &Let, result: &mut HashMap<Span, InternedString>) {
        result.insert(r#let.name_span, r#let.name);
        self.init_span_string_map_expr(&r#let.value, result);
    }

    pub fn init_span_string_map_func(&self, func: &Func, result: &mut HashMap<Span, InternedString>) {
        result.insert(func.name_span, func.name);

        for arg in func.args.iter() {
            result.insert(arg.name_span, arg.name);
        }

        for generic in func.generics.iter() {
            result.insert(generic.name_span, generic.name);
        }

        self.init_span_string_map_expr(&func.value, result);
    }

    pub fn init_span_string_map_assert(&self, assert: &Assert, result: &mut HashMap<Span, InternedString>) {
        self.init_span_string_map_expr(&assert.value, result);
    }

    pub fn init_span_string_map_expr(&self, expr: &Expr, result: &mut HashMap<Span, InternedString>) {
        match expr {
            Expr::Identifier(_) |
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } |
            Expr::Path { .. } |
            Expr::FieldModifier { .. } => {},

            Expr::If(r#if) => {
                self.init_span_string_map_expr(&r#if.cond, result);
                self.init_span_string_map_expr(&r#if.true_value, result);
                self.init_span_string_map_expr(&r#if.false_value, result);
            },
            Expr::Match(r#match) => todo!(),
            Expr::Block(block) => {
                for r#let in block.lets.iter() {
                    self.init_span_string_map_let(r#let, result);
                }

                for assert in block.asserts.iter() {
                    self.init_span_string_map_assert(assert, result);
                }

                self.init_span_string_map_expr(&block.value, result);
            },
            Expr::ShortCircuit { lhs, rhs, .. } => {
                self.init_span_string_map_expr(lhs, result);
                self.init_span_string_map_expr(rhs, result);
            },
            Expr::Call { func, args, .. } => {
                if let Callable::Dynamic(f) = func {
                    self.init_span_string_map_expr(f, result);
                }

                for arg in args.iter() {
                    self.init_span_string_map_expr(arg, result);
                }
            },
        }
    }
}
