use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{Expr, FuncArgDef, Pattern, StructField, Type, Use};
use sodigy_name_analysis::NameKind;
use sodigy_span::{RenderableSpan, Span};
use std::collections::HashMap;

mod endec;
mod session;

pub use session::Session;

impl Session {
    pub fn ingest(
        &mut self,
        module_span: Span,  // of this hir
        hir_session: sodigy_hir::Session,
    ) {
        for (def_span, (args, generics)) in hir_session.funcs.iter().map(
            |func| (
                func.name_span,
                (
                    func.args.iter().map(
                        |arg| FuncArgDef {
                            name: arg.name,
                            name_span: arg.name_span,
                            r#type: None,
                            default_value: arg.default_value,
                        }
                    ).collect(),
                    func.generics.clone(),
                ),
            )
        ) {
            self.func_shapes.insert(def_span, (args, generics));
        }

        for (def_span, (fields, generics)) in hir_session.structs.iter().map(
            |r#struct| (
                r#struct.name_span,
                (
                    r#struct.fields.iter().map(
                        |field| StructField {
                            name: field.name,
                            name_span: field.name_span,
                            r#type: None,
                            default_value: field.default_value,
                        }
                    ).collect(),
                    r#struct.generics.clone(),
                ),
            )
        ) {
            self.struct_shapes.insert(def_span, (fields, generics));
        }

        let mut children = HashMap::new();

        for (name, span, _) in hir_session.iter_public_names() {
            children.insert(name, span);
        }

        self.module_name_map.insert(
            module_span,
            (
                module_span,
                NameKind::Module,
                children,
            ),
        );
    }

    pub fn resolve(&mut self, hir_session: &mut sodigy_hir::Session) {
        self.name_aliases = HashMap::new();
        self.type_aliases = HashMap::new();

        for r#use in hir_session.uses.iter() {
            self.name_aliases.insert(r#use.name_span, r#use.clone());
        }

        for alias in hir_session.aliases.iter() {
            self.type_aliases.insert(alias.name_span, alias.clone());
        }

        self.resolve_alias_recursive();

        if !self.errors.is_empty() {
            return;
        }

        for r#let in hir_session.lets.iter_mut() {
            if let Some(r#type) = &mut r#let.r#type {
                self.resolve_type_recursive(r#type);
            }

            self.resolve_expr_recursive(&mut r#let.value);
        }

        for func in hir_session.funcs.iter_mut() {
            if let Some(r#type) = &mut func.r#type {
                self.resolve_type_recursive(r#type);
            }

            for arg in func.args.iter_mut() {
                if let Some(r#type) = &mut arg.r#type {
                    self.resolve_type_recursive(r#type);
                }
            }

            self.resolve_expr_recursive(&mut func.value);
        }

        // TODO: structs, enums, asserts
    }

    // If there's `use x as y;` and `use y as z;`, we have to
    // replace `use y as z;` with `use x as z;`.
    // Also, if there's `type MyInt = Int;` and `type YourInt = MyInt;`,
    // we have to replace `type YourInt = MyInt;` with `type YourInt = Int;`.
    pub fn resolve_alias_recursive(&mut self) {
        let mut emergency = false;

        // TODO: make recursion limit configurable
        'outer: for i in 0..65 {
            let mut nested_aliases = HashMap::new();
            let mut error_spans = None;

            // name_alias: `use y as z;`
            // def_span: `z` in `use y as z;`
            // name_alias.root: `y` in `use y as z;`
            // new_alias: `use x as y;`
            for (def_span, name_alias) in self.name_aliases.iter() {
                if let Some(new_alias) = self.name_aliases.get(&name_alias.root.def_span) {
                    nested_aliases.insert(*def_span, new_alias.clone());
                    error_spans = Some((*def_span, new_alias.name_span));
                }
            }

            if nested_aliases.is_empty() {
                break;
            }

            else if i == 64 || emergency {
                let (span1, span2) = error_spans.unwrap();
                self.errors.push(Error {
                    kind: ErrorKind::NameAliasRecursionLimitReached,
                    spans: vec![
                        RenderableSpan {
                            span: span1,
                            auxiliary: false,
                            note: None,
                        },
                        RenderableSpan {
                            span: span2,
                            auxiliary: false,
                            note: None,
                        },
                    ],
                    note: None,
                });
                break;
            }

            else {
                // old_alias: `use a.b.c as x;`
                // def_span: `x` in `use a.b.c as x;`
                // new_alias: `use d.e.f as a;`
                // we have to insert (def_span: x, alias: `use d.e.f.b.c as x;`)
                for (def_span, new_alias) in nested_aliases.iter() {
                    let old_alias: Use = self.name_aliases.get(def_span).unwrap().clone();
                    let new_fields = vec![
                        new_alias.fields.clone(),
                        old_alias.fields.clone(),
                    ].concat();

                    // If there's an infinite recursion, the length will increase exponentially.
                    // We have to escape!!
                    if new_fields.len() > 2048 {
                        emergency = true;
                    }

                    else if old_alias.name_span == new_alias.root.def_span {
                        self.errors.push(Error {
                            kind: ErrorKind::NameAliasRecursionLimitReached,
                            spans: vec![
                                RenderableSpan {
                                    span: old_alias.name_span,
                                    auxiliary: false,
                                    note: None,
                                },
                                RenderableSpan {
                                    span: old_alias.root.span,
                                    auxiliary: true,
                                    note: None,
                                },
                                RenderableSpan {
                                    span: new_alias.name_span,
                                    auxiliary: true,
                                    note: None,
                                },
                                RenderableSpan {
                                    span: new_alias.root.span,
                                    auxiliary: true,
                                    note: None,
                                },
                            ],
                            note: None,
                        });
                        break 'outer;
                    }

                    else {
                        self.name_aliases.insert(
                            *def_span,
                            Use {
                                keyword_span: old_alias.keyword_span,
                                name: old_alias.name,
                                name_span: old_alias.name_span,
                                fields: new_fields,
                                root: new_alias.root,
                            },
                        );
                    }
                }
            }
        }

        for i in 0..65 {
            // TODO
            // let mut nested_aliases = HashMap::new();

            // for (def_span, type_alias) in self.type_aliases.iter() {
            //     match &type_alias.r#type {
            //         Type::Identifier(id) | Type::Path { id, .. } => todo!(),
            //         Type::Param { r#type, .. } => todo!(),
            //         Type::Tuple { types, .. } => todo!(),
            //         Type::Func { args, r#return, .. } => todo!(),
            //         Type::Wildcard(_) => {},
            //     }
            // }

            // if nested_aliases.is_empty() {
            //     break;
            // }

            // else if i == 64 {
            //     self.errors.push(Error {
            //         kind: ErrorKind::TypeAliasRecursionLimitReached,
            //         spans: todo!(),
            //         note: None,
            //     });
            // }

            // else {
            //     // TODO: apply aliases
            //     todo!()
            // }
        }
    }

    pub fn resolve_type_recursive(&mut self, r#type: &mut Type) {
        // I just realized that we have to resolve name_aliases and type_aliases here
        todo!()
    }

    pub fn resolve_expr_recursive(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => {},
            Expr::Identifier(id) => match self.name_aliases.get(&id.def_span) {
                Some(origin) => {
                    if origin.fields.is_empty() {
                        *expr = Expr::Identifier(origin.root);
                    }

                    else {
                        *expr = Expr::Path {
                            lhs: Box::new(Expr::Identifier(origin.root)),
                            fields: origin.fields.clone(),
                        };
                    }
                },
                None => {},
            },
            Expr::Path { lhs, fields } => match &mut **lhs {
                Expr::Identifier(id) => match self.name_aliases.get(&id.def_span) {
                    Some(origin) => {
                        // There's `use a.b.c as x;` and `x.y.z`
                        // Then the origin is `Use { name: x, root: a, fields: [b, c] }` and
                        // the expr is `Expr::Path { lhs: x, fields: [y, z] }`
                        // It has to become `Expr::Path { lhs: a, fields: [b, c, y, z] }`
                        *lhs = Box::new(Expr::Identifier(origin.root));

                        if !origin.fields.is_empty() {
                            *fields = vec![
                                origin.fields.clone(),
                                fields.clone(),
                            ].concat();
                        }
                    },
                    None => {},
                },
                Expr::Path { .. } => panic!("ICE"),  // It should have been flattened
                e => {
                    self.resolve_expr_recursive(e);
                },
            },
            Expr::If(r#if) => {
                self.resolve_expr_recursive(&mut r#if.cond);

                if let Some(full_pattern) = &mut r#if.pattern {
                    self.resolve_pattern_recursive(&mut full_pattern.pattern);
                }

                self.resolve_expr_recursive(&mut r#if.true_value);
                self.resolve_expr_recursive(&mut r#if.false_value);
            },
            Expr::Match(r#match) => todo!(),
            Expr::Block(block) => todo!(),
            Expr::Call { func, args } => {
                self.resolve_expr_recursive(func);

                for arg in args.iter_mut() {
                    self.resolve_expr_recursive(&mut arg.arg);
                }
            },
            Expr::Tuple { elements, .. } | Expr::List { elements, .. } => {
                for element in elements.iter_mut() {
                    self.resolve_expr_recursive(element);
                }
            },
            Expr::StructInit { r#struct, fields, .. } => todo!(),
            Expr::FieldModifier { lhs, rhs, .. } |
            Expr::InfixOp { lhs, rhs, .. } => {
                self.resolve_expr_recursive(lhs);
                self.resolve_expr_recursive(rhs);
            },
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => {
                self.resolve_expr_recursive(hs);
            },
        }
    }

    pub fn resolve_pattern_recursive(&mut self, pattern: &mut Pattern) {
        todo!()
    }
}
