use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    Expr,
    FuncArgDef,
    Pattern,
    StructFieldDef,
    Type,
    Use,
};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
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
                        |field| StructFieldDef {
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

        for (name, span, _) in hir_session.iter_item_names() {
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

        for (name, span) in hir_session.lang_items.into_iter() {
            self.lang_items.insert(name, span);
        }
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

        self.resolve_alias();

        if !self.errors.is_empty() {
            return;
        }

        for r#let in hir_session.lets.iter_mut() {
            if let Some(r#type) = &mut r#let.r#type {
                self.resolve_type(r#type, &mut vec![]);
                self.resolve_name_alias_in_type(r#type, &mut vec![]);
            }

            self.resolve_expr(&mut r#let.value);
        }

        for func in hir_session.funcs.iter_mut() {
            if let Some(r#type) = &mut func.r#type {
                self.resolve_type(r#type, &mut vec![]);
                self.resolve_name_alias_in_type(r#type, &mut vec![]);
            }

            for arg in func.args.iter_mut() {
                if let Some(r#type) = &mut arg.r#type {
                    self.resolve_type(r#type, &mut vec![]);
                    self.resolve_name_alias_in_type(r#type, &mut vec![]);
                }
            }

            self.resolve_expr(&mut func.value);
        }

        // TODO: structs, enums, asserts

        let mut external_names: HashMap<Span, InternedString> = HashMap::new();

        for r#use in self.name_aliases.values() {
            if let NameOrigin::External = r#use.root.origin {
                external_names.insert(r#use.root.span, r#use.root.id);
            }
        }

        for (span, name) in external_names.iter() {
            self.errors.push(Error {
                kind: ErrorKind::UndefinedName(*name),
                spans: span.simple_error(),
                note: None,
            });
        }
    }

    // If there's `use x as y;` and `use y as z;`, we have to
    // replace `use y as z;` with `use x as z;`.
    // Also, if there's `type MyInt = Int;` and `type YourInt = MyInt;`,
    // we have to replace `type YourInt = MyInt;` with `type YourInt = Int;`.
    pub fn resolve_alias(&mut self) {
        let mut emergency = false;

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

                    continue;
                }

                // TODO: check type_alias in names
                // e.g. `type Foo = Bar; use Foo as Baz;`
                //      -> We have to replace `use Foo as Baz;` with `use Bar as Baz;`
            }

            // type_alias: `type Foo = Bar;`
            // def_span: `Foo` in `type Foo = Bar;`
            for (def_span, type_alias) in self.type_aliases.clone().iter() {
                let mut alias = type_alias.r#type.clone();
                let mut alias_log = vec![];
                self.resolve_type(&mut alias, &mut alias_log);

                if !alias_log.is_empty() {
                    nested_type_aliases.insert(*def_span, alias);

                    if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(*def_span);
                        suspicious_spans.extend(alias_log);
                    }

                    continue;
                }

                self.resolve_name_alias_in_type(&mut alias, &mut alias_log);

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
                                visibility: todo!(),
                                keyword_span: old_alias.keyword_span,
                                name: old_alias.name,
                                name_span: old_alias.name_span,
                                fields: new_fields,
                                root: new_alias.root,
                            },
                        );
                    }
                }

                for (def_span, new_alias) in nested_type_aliases.into_iter() {
                    match self.type_aliases.get_mut(&def_span) {
                        Some(old_alias) => {
                            old_alias.r#type = new_alias;
                        },
                        None => unreachable!(),
                    }
                }
            }
        }
    }

    // It resolves type aliases in a type annotation or a type alias.
    // `x: Option<Int>` -> here, `Option<Int>` is a type annotation.
    // `type MyOption = Option<Int>;` -> here, `Option<Int>` is a type alias.
    //
    // Let's say there's `type MyOption = Option<Int>;` and a type annotation `x: MyOption`.
    // Then it replaces `MyOption` in the type annotation with `Option<Int>`.
    pub fn resolve_type(
        &mut self,
        r#type: &mut Type,

        // If it resolves something, it pushes related spans to this vector.
        // The spans are used
        //    1) for error messages, when alias_resolve_recursion_limit is reached.
        //    2) to check whether anything has been resolved or not
        alias_log: &mut Vec<Span>,
    ) {
        match r#type {
            Type::Identifier(id) => match self.type_aliases.get(&id.def_span) {
                Some(alias) => {
                    // alias: type Bar = Foo;
                    // alias: type Bar = Foo<Int, Int>;
                    // alias: type Bar = (Int, Int);
                    // r#type: Bar
                    if alias.generics.is_empty() {
                        alias_log.push(id.def_span);
                        alias_log.push(id.span);
                        *r#type = alias.r#type.clone();
                    }

                    // alias: type Bar<T> = Foo<T>;
                    // r#type: Bar
                    else {
                        self.errors.push(Error {
                            kind: ErrorKind::MissingTypeArgument {
                                expected: alias.generics.len(),
                                got: 0,
                            },
                            spans: vec![
                                RenderableSpan {
                                    span: alias.group_span.unwrap(),
                                    auxiliary: true,
                                    note: Some(format!(
                                        "It requires {} argument{}.",
                                        alias.generics.len(),
                                        if alias.generics.len() == 1 { "" } else { "s" },
                                    )),
                                },
                                RenderableSpan {
                                    span: id.span,
                                    auxiliary: false,
                                    note: Some(String::from("There're no arguments.")),
                                },
                            ],
                            note: None,
                        });
                    }
                },
                None => {},
            },
            Type::Path { id, fields } => match self.type_aliases.get(&id.def_span) {
                Some(alias) => {
                    if alias.generics.is_empty() {
                        match &alias.r#type {
                            // alias: type Bar = Foo;
                            // r#type: Bar.a.b
                            Type::Identifier(alias_id) => {
                                // I'm not sure whether this case is compatible with the current type system.
                                // If so, the type-checker will handle this. There's no need to worry about it.
                                *r#type = Type::Path { id: *alias_id, fields: fields.clone() };
                                alias_log.push(alias_id.def_span);
                                alias_log.push(alias_id.span);
                            },
                            // alias: type Bar = Foo.a.b;
                            // r#type: Bar.c.d.
                            Type::Path { id: alias_id, fields: alias_fields } => {
                                // This doesn't make sense, and the type-checker will reject this.
                                // It's not resolver's responsibility to catch this error here.
                                *r#type = Type::Path {
                                    id: *alias_id,
                                    fields: vec![
                                        alias_fields.clone(),
                                        fields.clone(),
                                    ].concat(),
                                };
                                alias_log.push(alias_id.def_span);
                                alias_log.push(alias_id.span);
                            },
                            // alias: type Bar = Option<Int>;
                            // r#type: Bar.Some   -> this is not a valid type annotation
                            //
                            // alias: type Bar = (Int, Int);
                            // alias: type Bar = Fn(Int, Int) -> Int;
                            // alias: type Bar = _;
                            // r#type: Bar.a.b
                            Type::Param { .. } |
                            Type::Tuple { .. } |
                            Type::Func { .. } |
                            Type::Wildcard(_) |
                            Type::Never(_) => {
                                self.errors.push(todo!());
                            },
                        }
                    }

                    else {
                        // alias: type Bar<T> = Foo;
                        // alias: type Bar<T> = Foo.c.d;
                        // alias: type Bar<T> = (Int, Int);
                        // alias: type Bar<T> = Fn(Int, Int) -> Int;
                        // alias: type Bar<T> = _;
                        // r#type: Bar.a.b
                        //
                        // Regardless of the rhs of the type alias, it's an error because it's missing the argument.
                        self.errors.push(Error {
                            kind: ErrorKind::MissingTypeArgument {
                                expected: alias.generics.len(),
                                got: 0,
                            },
                            spans: vec![
                                RenderableSpan {
                                    span: alias.group_span.unwrap(),
                                    auxiliary: true,
                                    note: Some(format!(
                                        "It requires {} argument{}.",
                                        alias.generics.len(),
                                        if alias.generics.len() == 1 { "" } else { "s" },
                                    )),
                                },
                                RenderableSpan {
                                    span: id.span,
                                    auxiliary: false,
                                    note: Some(String::from("There're no arguments.")),
                                },
                            ],
                            note: None,
                        });
                    }
                },
                None => {},
            },
            Type::Param { r#type: type_p, args, group_span } => match &**type_p {
                Type::Identifier(id) => match self.type_aliases.get(&id.def_span) {
                    Some(alias) => {
                        if alias.generics.is_empty() {
                            match &alias.r#type {
                                // alias: type Bar = Foo;
                                // r#type: Bar<T, U>
                                Type::Identifier(alias_id) => {
                                    alias_log.push(id.def_span);
                                    alias_log.push(id.span);
                                    *r#type = Type::Param {
                                        r#type: Box::new(Type::Identifier(*alias_id)),
                                        args: args.clone(),
                                        group_span: *group_span,
                                    };
                                },
                                // alias: type Bar = Foo.a.b;
                                // r#type: Bar<T, U>
                                Type::Path { .. } => {
                                    alias_log.push(id.def_span);
                                    alias_log.push(id.span);
                                    *r#type = Type::Param {
                                        r#type: Box::new(alias.r#type.clone()),
                                        args: args.clone(),
                                        group_span: *group_span,
                                    };
                                },
                                // alias: type Bar = Foo<Int, Int>;
                                // alias: type Bar = (Int, Int);
                                // alias: type Bar = Fn(Int, Int) -> Int;
                                // alias: type Bar = _;
                                // alias: type Bar = !;
                                // r#type: Bar<T, U>
                                Type::Param { .. } |
                                Type::Tuple { .. } |
                                Type::Func { .. } |
                                Type::Wildcard(_) |
                                Type::Never(_) => {
                                    self.errors.push(Error {
                                        kind: ErrorKind::UnexpectedTypeArgument {
                                            expected: 0,
                                            got: args.len(),
                                        },
                                        spans: vec![
                                            RenderableSpan {
                                                span: alias.name_span,
                                                auxiliary: true,
                                                note: Some(String::from("It requires no arguments.")),
                                            },
                                            RenderableSpan {
                                                span: *group_span,
                                                auxiliary: false,
                                                note: Some(format!(
                                                    "It has {} unnecessary argument{}.",
                                                    args.len(),
                                                    if args.len() == 1 { "" } else { "s" },
                                                )),
                                            },
                                        ],
                                        note: None,
                                    });
                                },
                            }
                        }

                        else {
                            match &alias.r#type {
                                // alias: type Bar<T, U> = Foo;
                                // alias: type Bar<T, U> = Foo.a.b;
                                // alias: type Bar<T, U> = _;
                                // alais: type Bar<T, U> = !;
                                // r#type: Bar<T, U>
                                Type::Identifier(_) |
                                Type::Path { .. } |
                                Type::Wildcard(_) |
                                Type::Never(_) => {
                                    // This is very very strange and meaningless, but not an error anyway.
                                    if alias.generics.len() == args.len() {
                                        match &alias.r#type {
                                            Type::Identifier(alias_id) |
                                            Type::Path { id: alias_id, .. } => {
                                                alias_log.push(alias_id.span);
                                                alias_log.push(alias_id.def_span);
                                            },
                                            Type::Wildcard(span) => {
                                                alias_log.push(*span);
                                            },
                                            _ => unreachable!(),
                                        }

                                        *r#type = alias.r#type.clone();
                                    }

                                    else {
                                        let error_kind = if alias.generics.len() > args.len() {
                                            ErrorKind::MissingTypeArgument {
                                                expected: alias.generics.len(),
                                                got: args.len(),
                                            }
                                        } else {
                                            ErrorKind::UnexpectedTypeArgument {
                                                expected: alias.generics.len(),
                                                got: args.len(),
                                            }
                                        };
                                        self.errors.push(Error {
                                            kind: error_kind,
                                            spans: vec![
                                                RenderableSpan {
                                                    span: alias.group_span.unwrap(),
                                                    auxiliary: true,
                                                    note: Some(format!(
                                                        "It requires {} argument{}.",
                                                        alias.generics.len(),
                                                        if alias.generics.len() == 1 { "" } else { "s" },
                                                    )),
                                                },
                                                RenderableSpan {
                                                    span: *group_span,
                                                    auxiliary: false,
                                                    note: Some(format!(
                                                        "It has {} argument{}.",
                                                        args.len(),
                                                        if args.len() == 1 { "" } else { "s" },
                                                    )),
                                                },
                                            ],
                                            note: None,
                                        });
                                    }
                                },
                                // alias: type Bar<T, U> = Foo<T, U>;
                                // alias: type Bar<T, U> = (T, U);
                                // alias: type Bar<T, U> = Fn(T) -> U;
                                // r#type: Bar<Int, Int>
                                Type::Param { .. } |
                                Type::Tuple { .. } |
                                Type::Func { .. } => {
                                    if alias.generics.len() == args.len() {
                                        // clone the alias_type and replace `T` and `U` with `Int` and `Int`.
                                        todo!()
                                    }

                                    else {
                                        let error_kind = if alias.generics.len() > args.len() {
                                            ErrorKind::MissingTypeArgument {
                                                expected: alias.generics.len(),
                                                got: args.len(),
                                            }
                                        } else {
                                            ErrorKind::UnexpectedTypeArgument {
                                                expected: alias.generics.len(),
                                                got: args.len(),
                                            }
                                        };
                                        self.errors.push(Error {
                                            kind: error_kind,
                                            spans: vec![
                                                RenderableSpan {
                                                    span: alias.group_span.unwrap(),
                                                    auxiliary: true,
                                                    note: Some(format!(
                                                        "It requires {} argument{}.",
                                                        alias.generics.len(),
                                                        if alias.generics.len() == 1 { "" } else { "s" },
                                                    )),
                                                },
                                                RenderableSpan {
                                                    span: *group_span,
                                                    auxiliary: false,
                                                    note: Some(format!(
                                                        "It has {} argument{}.",
                                                        args.len(),
                                                        if args.len() == 1 { "" } else { "s" },
                                                    )),
                                                },
                                            ],
                                            note: None,
                                        });
                                    }
                                },
                            }
                        }
                    },
                    None => {},
                },
                Type::Path { id, .. } => match self.type_aliases.get(&id.def_span) {
                    Some(alias) => {
                        // alias: type Bar = ???;
                        // r#type: Bar.a.b<T, U>
                        if alias.generics.is_empty() {
                            todo!()
                        }

                        // alias: type Bar<T, U> = ???;
                        // r#type: Bar.a.b<T, U>
                        else {
                            todo!()
                        }
                    },
                    None => {},
                },
                _ => unreachable!(),
            },
            Type::Tuple { types, .. } => {
                for r#type in types.iter_mut() {
                    self.resolve_type(r#type, alias_log);
                }
            },
            Type::Func { r#return, args, .. } => {
                self.resolve_type(r#return, alias_log);

                for arg in args.iter_mut() {
                    self.resolve_type(arg, alias_log);
                }
            },
            Type::Wildcard(_) | Type::Never(_) => {},
        }
    }

    // It resolves name aliases in expressions, recursively.
    // For example, if there's `use Foo.Bar as x;` and an expression `x + 1`,
    // it replaces the expression with `Foo.Bar + 1`.
    // There should be no further alias in `use Foo.Bar as x;` because
    // `resolve_alias` already resolved all the aliases in aliases.
    pub fn resolve_expr(&mut self, expr: &mut Expr) {
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
                Expr::Path { .. } => unreachable!(),  // It should have been flattened
                e => {
                    self.resolve_expr(e);
                },
            },
            Expr::If(r#if) => {
                self.resolve_expr(&mut r#if.cond);

                if let Some(full_pattern) = &mut r#if.pattern {
                    self.resolve_pattern(&mut full_pattern.pattern);
                }

                self.resolve_expr(&mut r#if.true_value);
                self.resolve_expr(&mut r#if.false_value);
            },
            Expr::Match(r#match) => todo!(),
            Expr::Block(block) => todo!(),
            Expr::Call { func, args } => {
                self.resolve_expr(func);

                for arg in args.iter_mut() {
                    self.resolve_expr(&mut arg.arg);
                }
            },
            Expr::Tuple { elements, .. } | Expr::List { elements, .. } => {
                for element in elements.iter_mut() {
                    self.resolve_expr(element);
                }
            },
            Expr::StructInit { r#struct, fields, .. } => todo!(),
            Expr::FieldModifier { lhs, rhs, .. } |
            Expr::InfixOp { lhs, rhs, .. } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            },
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => {
                self.resolve_expr(hs);
            },
        }
    }

    // Let's say there's `x: [MyChar]; use foo.bar.Char as MyChar;`.
    // Then, it replaces `MyChar` in the type annotation with `foo.bar.Char`. So it
    // becomes `x: [foo.bar.Char];`
    pub fn resolve_name_alias_in_type(
        &mut self,
        r#type: &mut Type,

        // If it resolves something, it pushes related spans to this vector.
        // The spans are used
        //    1) for error messages, when alias_resolve_recursion_limit is reached.
        //    2) to check whether anything has been resolved or not
        alias_log: &mut Vec<Span>,
    ) {
        match r#type {
            Type::Identifier(id) => match self.name_aliases.get(&id.def_span) {
                Some(alias) => {
                    alias_log.push(id.def_span);
                    alias_log.push(id.span);

                    // type: `x: MyChar`
                    // alias: `use Char as MyChar;`
                    if alias.fields.is_empty() {
                        *r#type = Type::Identifier(alias.root);
                    }

                    // type: `x: MyChar`
                    // alias: `use foo.bar.Char as MyChar;`
                    else {
                        *r#type = Type::Path {
                            id: alias.root,
                            fields: alias.fields.clone(),
                        };
                    }
                },
                None => {},
            },
            Type::Path { id, fields } => match self.name_aliases.get(&id.def_span) {
                Some(alias) => {
                    alias_log.push(id.def_span);
                    alias_log.push(id.span);

                    // type: `x: foo.bar.Char`
                    // alias: `use baz as foo`
                    //   -> `x: baz.bar.Char`
                    //
                    // type: `x: foo.bar.Char`
                    // alias: `use baz.goo as foo;`
                    //   -> `x: baz.goo.bar.Char`
                    *r#type = Type::Path {
                        id: alias.root,
                        fields: vec![
                            alias.fields.clone(),
                            fields.clone(),
                        ].concat(),
                    };
                },
                None => {},
            },
            Type::Param { r#type: p_type, args, .. } => {
                self.resolve_name_alias_in_type(p_type, alias_log);

                for arg in args.iter_mut() {
                    self.resolve_name_alias_in_type(arg, alias_log);
                }
            },
            Type::Tuple { types, .. } => {
                for r#type in types.iter_mut() {
                    self.resolve_name_alias_in_type(r#type, alias_log);
                }
            },
            Type::Func { args, r#return, .. } => {
                self.resolve_name_alias_in_type(r#return, alias_log);

                for arg in args.iter_mut() {
                    self.resolve_name_alias_in_type(arg, alias_log);
                }
            },
            Type::Wildcard(_) | Type::Never(_) => {},
        }
    }

    pub fn resolve_pattern(&mut self, pattern: &mut Pattern) {
        todo!()
    }
}
