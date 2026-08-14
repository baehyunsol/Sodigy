use super::Monomorphization;
use crate::Session;
use sodigy_error::TypeVarInfo;
use sodigy_mir::{Callable, Expr, MacroKind, Type};
use sodigy_parse::Field;
use sodigy_span::Span;
use std::collections::HashSet;

impl Session {
    pub fn monomorphize_expr(
        &mut self,
        expr: &mut Expr,
        wildcard_spans: &HashSet<Span>,
        monomorphization: &Monomorphization,
    ) {
        match expr {
            Expr::Ident { id, dotfish } => {
                self.monomorphize_id(id, monomorphization);
                self.monomorphize_dotfish(dotfish, wildcard_spans, monomorphization);
            },
            Expr::Constant(c) => {
                *c = c.monomorphize(monomorphization.id);
            },
            Expr::If(r#if) => {
                r#if.if_span = r#if.if_span.monomorphize(monomorphization.id);
                r#if.else_span = r#if.else_span.monomorphize(monomorphization.id);
                r#if.true_group_span = r#if.true_group_span.monomorphize(monomorphization.id);
                r#if.false_group_span = r#if.false_group_span.monomorphize(monomorphization.id);
                self.monomorphize_expr(&mut r#if.cond, wildcard_spans, monomorphization);
                self.monomorphize_expr(&mut r#if.true_value, wildcard_spans, monomorphization);
                self.monomorphize_expr(&mut r#if.false_value, wildcard_spans, monomorphization);
            },
            Expr::Match(r#match) => {
                r#match.keyword_span = r#match.keyword_span.monomorphize(monomorphization.id);
                r#match.group_span = r#match.group_span.monomorphize(monomorphization.id);
                self.monomorphize_expr(&mut r#match.scrutinee, wildcard_spans, monomorphization);

                for arm in r#match.arms.iter_mut() {
                    self.monomorphize_pattern(&mut arm.pattern, monomorphization);
                    self.monomorphize_expr(&mut arm.value, wildcard_spans, monomorphization);

                    if let Some(guard) = &mut arm.guard {
                        self.monomorphize_expr(guard, wildcard_spans, monomorphization);
                    }
                }
            },
            Expr::Block(block) => {
                block.group_span = block.group_span.monomorphize(monomorphization.id);
                self.monomorphize_expr(&mut block.value, wildcard_spans, monomorphization);

                for r#let in block.lets.iter_mut() {
                    let new_name_span = r#let.name_span.monomorphize(monomorphization.id);
                    let old_type = match self.types.get(&r#let.name_span) {
                        Some(r#type) => r#type.clone(),
                        None => {
                            let type_var = Type::Var { def_span: new_name_span.clone(), is_return: false };
                            self.add_type_var(type_var.clone(), Some(TypeVarInfo::Ident(r#let.name)));
                            type_var
                        },
                    };

                    let new_type = self.monomorphize_type(&old_type, wildcard_spans, monomorphization);
                    self.types.insert(new_name_span.clone(), new_type);
                    r#let.keyword_span = r#let.keyword_span.monomorphize(monomorphization.id);
                    r#let.name_span = new_name_span.clone();
                    r#let.type_annot_span = r#let.type_annot_span.as_ref().map(|span| span.monomorphize(monomorphization.id));
                    self.monomorphize_expr(&mut r#let.value, wildcard_spans, monomorphization);
                    // TODO: do we have to change `LetOrigin`?
                }

                for assert in block.asserts.iter_mut() {
                    assert.keyword_span = assert.keyword_span.monomorphize(monomorphization.id);
                    self.monomorphize_expr(&mut assert.value, wildcard_spans, monomorphization);

                    if let Some(note) = &mut assert.note {
                        self.monomorphize_expr(note, wildcard_spans, monomorphization);
                    }

                    if let Some(note_decorator_span) = &mut assert.note_decorator_span {
                        *note_decorator_span = note_decorator_span.monomorphize(monomorphization.id);
                    }
                }
            },
            Expr::Field { lhs, fields, dotfish } => {
                self.monomorphize_expr(lhs, wildcard_spans, monomorphization);

                for field in fields.iter_mut() {
                    if let Field::Name { name_span, dot_span, .. } = field {
                        *name_span = name_span.monomorphize(monomorphization.id);
                        *dot_span = dot_span.monomorphize(monomorphization.id);
                    }
                }

                for d in dotfish.iter_mut() {
                    self.monomorphize_dotfish(d, wildcard_spans, monomorphization);
                }
            },
            Expr::FieldUpdate { lhs, fields, rhs } => {
                self.monomorphize_expr(lhs, wildcard_spans, monomorphization);

                for field in fields.iter_mut() {
                    if let Field::Name { name_span, dot_span, .. } = field {
                        *name_span = name_span.monomorphize(monomorphization.id);
                        *dot_span = dot_span.monomorphize(monomorphization.id);
                    }
                }

                self.monomorphize_expr(rhs, wildcard_spans, monomorphization);
            },
            Expr::Call { func, args, arg_group_span, types, .. } => {
                match func {
                    Callable::Static { span, .. } |
                    Callable::StructInit { span, .. } |
                    Callable::EnumInit { span, .. } |
                    Callable::TupleInit { group_span: span, .. } |
                    Callable::ListInit { group_span: span, .. } => {
                        *span = span.monomorphize(monomorphization.id);
                    },
                    Callable::Dynamic(c) => {
                        self.monomorphize_expr(c, wildcard_spans, monomorphization);
                    },
                }

                for arg in args.iter_mut() {
                    self.monomorphize_expr(arg, wildcard_spans, monomorphization);
                }

                *arg_group_span = arg_group_span.monomorphize(monomorphization.id);
                self.monomorphize_dotfish(types, wildcard_spans, monomorphization);
            },
            Expr::Macro { kind, macro_span, group_span } => {
                *macro_span = macro_span.monomorphize(monomorphization.id);
                *group_span = group_span.monomorphize(monomorphization.id);

                match &mut **kind {
                    MacroKind::IncludeString { .. } |
                    MacroKind::IncludeBytes { .. } |
                    MacroKind::File |
                    MacroKind::ModulePath |
                    MacroKind::Line |
                    MacroKind::Column => {},
                    MacroKind::TypeName { r#type } |
                    MacroKind::NumberOfVariants { r#type } |
                    MacroKind::NumberOfFields { r#type } |
                    MacroKind::NameOfVariants { r#type } |
                    MacroKind::NameOfFields { r#type } => {
                        *r#type = self.monomorphize_type(r#type, wildcard_spans, monomorphization);
                    },
                    MacroKind::TypeNameOfValue { value } => {
                        self.monomorphize_expr(value, wildcard_spans, monomorphization);
                    },
                }
            },
        }
    }
}
