use crate::Session;
use sodigy_hir::EnumVariantFields;
use sodigy_mir::{Assert, Callable, Enum, Expr, Func, Let, Struct};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Session {
    pub fn init_span_string_map(
        &mut self,
        lets: &[Let],
        funcs: &[Func],
        structs: &[Struct],
        enums: &[Enum],
        asserts: &[Assert],
        aliases: &[(InternedString, Span)],
    ) {
        let mut result = HashMap::new();

        for r#let in lets.iter() {
            self.init_span_string_map_let(r#let, &mut result);
        }

        for func in funcs.iter() {
            self.init_span_string_map_func(func, &mut result);
        }

        for r#struct in structs.iter() {
            self.init_span_string_map_struct(r#struct, &mut result);
        }

        for r#enum in enums.iter() {
            self.init_span_string_map_enum(r#enum, &mut result);
        }

        for assert in asserts.iter() {
            self.init_span_string_map_assert(assert, &mut result);
        }

        for (name, span) in aliases.iter() {
            result.insert(*span, *name);
        }

        self.span_string_map = result;
    }

    pub fn init_span_string_map_let(&self, r#let: &Let, result: &mut HashMap<Span, InternedString>) {
        result.insert(r#let.name_span, r#let.name);
        self.init_span_string_map_expr(&r#let.value, result);
    }

    pub fn init_span_string_map_func(&self, func: &Func, result: &mut HashMap<Span, InternedString>) {
        result.insert(func.name_span, func.name);

        for param in func.params.iter() {
            result.insert(param.name_span, param.name);
        }

        for generic in func.generics.iter() {
            result.insert(generic.name_span, generic.name);
        }

        self.init_span_string_map_expr(&func.value, result);
    }

    pub fn init_span_string_map_enum(&self, r#enum: &Enum, result: &mut HashMap<Span, InternedString>) {
        result.insert(r#enum.name_span, r#enum.name);

        for variant in r#enum.variants.iter() {
            result.insert(variant.name_span, variant.name);

            if let EnumVariantFields::Struct(fields) = &variant.fields {
                for field in fields.iter() {
                    result.insert(field.name_span, field.name);
                }
            }
        }

        for generic in r#enum.generics.iter() {
            result.insert(generic.name_span, generic.name);
        }
    }

    pub fn init_span_string_map_struct(&self, r#struct: &Struct, result: &mut HashMap<Span, InternedString>) {
        result.insert(r#struct.name_span, r#struct.name);

        for (name, name_span) in r#struct.fields.iter() {
            result.insert(*name_span, *name);
        }

        for generic in r#struct.generics.iter() {
            result.insert(generic.name_span, generic.name);
        }
    }

    pub fn init_span_string_map_assert(&self, assert: &Assert, result: &mut HashMap<Span, InternedString>) {
        self.init_span_string_map_expr(&assert.value, result);
    }

    pub fn init_span_string_map_expr(&self, expr: &Expr, result: &mut HashMap<Span, InternedString>) {
        match expr {
            Expr::Ident(_) | Expr::Constant(_) => {},
            Expr::Field { lhs, .. } => {
                self.init_span_string_map_expr(lhs, result);
            },
            Expr::FieldUpdate { lhs, rhs, .. } => {
                self.init_span_string_map_expr(lhs, result);
                self.init_span_string_map_expr(rhs, result);
            },
            Expr::If(r#if) => {
                self.init_span_string_map_expr(&r#if.cond, result);
                self.init_span_string_map_expr(&r#if.true_value, result);
                self.init_span_string_map_expr(&r#if.false_value, result);
            },
            Expr::Match(r#match) => {
                self.init_span_string_map_expr(&r#match.scrutinee, result);

                for arm in r#match.arms.iter() {
                    if let Some(guard) = &arm.guard {
                        self.init_span_string_map_expr(guard, result);
                    }

                    self.init_span_string_map_expr(&arm.value, result);
                }
            },
            Expr::Block(block) => {
                for r#let in block.lets.iter() {
                    self.init_span_string_map_let(r#let, result);
                }

                for assert in block.asserts.iter() {
                    self.init_span_string_map_assert(assert, result);
                }

                self.init_span_string_map_expr(&block.value, result);
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
