use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    Expr,
    FuncArgDef,
    Pattern,
    StructField,
    Type,
    Use,
};
use sodigy_name_analysis::NameKind;
use sodigy_span::{RenderableSpan, Span};
use std::collections::{HashMap, HashSet};

mod endec;
mod session;

pub use session::Session;

// TODO: make it configurable
const ALIAS_RESOLVE_RECURSION_LIMIT: usize = 64;

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

        // TODO: name <-> name (done)
        // TODO: type <-> type
        // TODO: type <-> name
        'outer: for i in 0..(ALIAS_RESOLVE_RECURSION_LIMIT + 1) {
            let mut nested_name_aliases: HashMap<Span, Use> = HashMap::new();
            let mut nested_type_aliases: HashMap<Span, Type> = HashMap::new();
            let mut suspicious_spans = vec![];

            // name_alias: `use y as z;`
            // def_span: `z` in `use y as z;`
            // name_alias.root: `y` in `use y as z;`
            // new_alias: `use x as y;`
            for (def_span, name_alias) in self.name_aliases.iter() {
                if let Some(new_alias) = self.name_aliases.get(&name_alias.root.def_span) {
                    nested_name_aliases.insert(*def_span, new_alias.clone());

                    if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(*def_span);
                        suspicious_spans.push(new_alias.name_span);
                    }
                }
            }

            // type_alias: `type Foo = Bar;`
            // def_span: `Foo` in `type Foo = Bar;`
            for (def_span, type_alias) in self.type_aliases.clone().iter() {
                let mut alias = type_alias.r#type.clone();
                let mut alias_log = vec![];
                self.resolve_type_recursive(&mut alias, &mut alias_log);

                if !alias_log.is_empty() {
                    nested_type_aliases.insert(*def_span, alias);

                    if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(*def_span);
                        suspicious_spans.extend(alias_log);
                    }
                }
            }

            if nested_name_aliases.is_empty() && nested_type_aliases.is_empty() {
                break;
            }

            else if i == ALIAS_RESOLVE_RECURSION_LIMIT || emergency {
                // dedup
                suspicious_spans = suspicious_spans.into_iter().collect::<HashSet<_>>().into_iter().collect();
                self.errors.push(Error {
                    kind: ErrorKind::AliasResolveRecursionLimitReached,
                    spans: suspicious_spans.iter().map(
                        |span| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect(),
                    note: Some(String::from("It seems like these names are aliases of each other.")),
                });
                break;
            }

            else {
                // old_alias: `use a.b.c as x;`
                // def_span: `x` in `use a.b.c as x;`
                // new_alias: `use d.e.f as a;`
                // we have to insert (def_span: x, alias: `use d.e.f.b.c as x;`)
                for (def_span, new_alias) in nested_name_aliases.iter() {
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
                            kind: ErrorKind::AliasResolveRecursionLimitReached,
                            spans: [old_alias.name_span, old_alias.root.span, new_alias.name_span, new_alias.root.span].iter().map(
                                |span| RenderableSpan {
                                    span: *span,
                                    auxiliary: false,
                                    note: None,
                                }
                            ).collect(),
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

                for (def_span, new_alias) in nested_type_aliases.iter() {
                    todo!()
                }
            }
        }
    }

    pub fn resolve_type_recursive(
        &mut self,
        r#type: &mut Type,

        // If it resolves something, it pushes related spans to this vector.
        // The spans are solely used for error messages, when alias_resolve_recursion_limit is reached.
        error_spans: &mut Vec<Span>,
    ) {
        match r#type {
            Type::Identifier(id) => match self.type_aliases.get(&id.def_span) {
                Some(alias) => {
                    // alias: type Bar = Foo;
                    // r#type: Bar
                    if alias.generics.is_empty() {
                        *r#type = alias.r#type.clone();
                    }

                    // alias: type Bar<T> = Foo<T>;
                    // r#type: Bar
                    else {
                        self.errors.push();
                    }
                },
                None => {},
            },
            Type::Path { id, .. } => match self.type_aliases.get(&id.def_span) {
                Some(alias) => {
                    // alias: type Bar = Foo;
                    // r#type: Bar.x.y
                    if alias.generics.is_empty() {
                        // It also depends on how the rhs of the alias looks like
                        todo!()
                    }

                    // alias type Bar<T> = Foo<T>;
                    // r#type: Bar.x.y
                    else {
                        // It also depends on how the rhs of the alias looks like
                        todo!()
                    }
                },
                None => {},
            },
            Type::Param { r#type, args, .. } => match r#type {
                Type::Identifier(id) => match self.type_aliases.get(&id.def_span) {
                    _ => todo!(),
                },
                Type::Path { id, .. } => match self.type_aliases.get(&id.def_span) {
                    _ => todo!(),
                },
                _ => panic!("ICE"),
            },
            Type::Tuple { types, .. } => {
                for r#type in types.iter_mut() {
                    self.resolve_type_recursive(r#type, error_spans);
                }
            },
            Type::Func { r#return, args, .. } => {
                self.resolve_type_recursive(r#return, error_spans);

                for arg in args.iter_mut() {
                    self.resolve_type_recursive(arg, error_spans);
                }
            },
            Type::Wildcard(_) => {},
        }
    }

    // TODO: it also has to resolve type_alias
    //       e.g. type MyOption<T> = Option<T>; let x = MyOption.Some(3);
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
