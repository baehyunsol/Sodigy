use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    Alias,
    Assert,
    Expr,
    Func,
    FuncArgDef,
    Let,
    Pattern,
    Session as HirSession,
    StructFieldDef,
    Type,
    Use,
};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, unintern_string};
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
        mut hir_session: sodigy_hir::Session,
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

        for (name, span, kind) in hir_session.iter_item_names() {
            children.insert(name, (span, kind));
        }

        self.module_name_map.insert(
            module_span,
            (
                NameKind::Module,
                children,
            ),
        );

        for (name, span) in hir_session.lang_items.into_iter() {
            self.lang_items.insert(name, span);
        }

        for r#use in hir_session.uses.drain(..) {
            self.name_aliases.insert(r#use.name_span, r#use);
        }

        for alias in hir_session.aliases.drain(..) {
            self.type_aliases.insert(alias.name_span, alias);
        }
    }

    // Aliases might be nested. e.g.
    // `type x = foo;`
    // `use x as y;`
    // `use y as z;`
    //
    // Then, we have to resolve the above aliases to
    // `type x = foo;`
    // `type foo as y;`
    // `type foo as z;`
    //
    // We have to do this before resolving aliases in expressions and type annotations.
    // We have to do this globally.
    // Also, there can be an infinite loop, so we have to set some kinda recursion limit.
    pub fn resolve_alias(&mut self) -> Result<(), ()> {
        let mut nested_name_aliases = HashMap::new();
        let mut nested_type_aliases = HashMap::new();
        let mut suspicious_spans = vec![];
        let mut has_error = false;

        for i in 0..(ALIAS_RESOLVE_RECURSION_LIMIT + 1) {
            let mut emergency_escape = false;

            for (name_span, mut alias) in self.type_aliases.clone().into_iter() {
                let mut alias_log = vec![];

                if let Err(()) = self.resolve_type(&mut alias.r#type, &mut alias_log) {
                    has_error = true;
                }

                if !alias_log.is_empty() {
                    if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(name_span);
                        suspicious_spans.extend(alias_log);
                    }

                    nested_type_aliases.insert(name_span, alias.r#type);
                }
            }

            for (name_span, mut r#use) in self.name_aliases.clone().into_iter() {
                let mut alias_log = vec![];

                if let Err(()) = self.resolve_use(&mut r#use, &mut alias_log) {
                    has_error = true;
                }

                if !alias_log.is_empty() {
                    // `use x.a.b as y;`
                    // `use y.c.d as x;`
                    // -> When you resolve this `n` times, the length of the
                    //    field will be `2^n`.
                    if r#use.fields.len() > 1024 {
                        suspicious_spans = alias_log;
                        suspicious_spans.push(name_span);
                        emergency_escape = true;
                    }

                    else if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(name_span);
                        suspicious_spans.extend(alias_log);
                    }

                    nested_name_aliases.insert(name_span, r#use);
                }
            }

            if i == ALIAS_RESOLVE_RECURSION_LIMIT || emergency_escape {
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
                return Err(());
            }

            else if has_error {
                return Err(());
            }

            else if !nested_name_aliases.is_empty() || !nested_type_aliases.is_empty() {
                for (name_span, r#use) in nested_name_aliases.drain() {
                    self.name_aliases.insert(name_span, r#use);
                }

                for (name_span, alias) in nested_type_aliases.drain() {
                    let old_alias = self.type_aliases.get_mut(&name_span).unwrap();
                    old_alias.r#type = alias;
                }
            }

            else {
                break;
            }
        }

        Ok(())
    }

    pub fn resolve_module(&mut self, hir_session: &mut HirSession) -> Result<(), ()> {
        let mut has_error = false;

        for r#let in hir_session.lets.iter_mut() {
            if let Err(()) = self.resolve_let(r#let) {
                has_error = true;
            }
        }

        for func in hir_session.funcs.iter_mut() {
            if let Err(()) = self.resolve_func(func) {
                has_error = true;
            }
        }

        // TODO: structs, enums

        for assert in hir_session.asserts.iter_mut() {
            if let Err(()) = self.resolve_assert(assert) {
                has_error = true;
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_let(&mut self, r#let: &mut Let) -> Result<(), ()> {
        let mut has_error = false;

        if let Some(r#type) = &mut r#let.r#type {
            if let Err(()) = self.resolve_type(r#type, &mut vec![]) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut r#let.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_func(&mut self, func: &mut Func) -> Result<(), ()> {
        let mut has_error = false;

        for arg in func.args.iter_mut() {
            if let Some(r#type) = &mut arg.r#type {
                if let Err(()) = self.resolve_type(r#type, &mut vec![]) {
                    has_error = true;
                }
            }
        }

        if let Some(r#type) = &mut func.r#type {
            if let Err(()) = self.resolve_type(r#type, &mut vec![]) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut func.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_assert(&mut self, assert: &mut Assert) -> Result<(), ()> {
        let mut has_error = false;

        if let Some(note) = &mut assert.note {
            if let Err(()) = self.resolve_expr(note) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut assert.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    // If `x` in `use x.y.z as w;` is an alias, it resolves `x`.
    // If `x` is a module in `use x.y as w;`, it finds the def_span of `y` and
    // replaces the alias with `use y as w;`
    //
    // There may be multiple levels of aliases in `use`. This function only resolves
    // one level of alias. `resolve_alias` will call this function multiple times
    // until all the aliases are resolved (or an `AliasResolveRecursionLimitReached` error).
    //
    // `log` does 2 things:
    //     1. It tells whether the function has resolved something. If `log` is not empty, something has happened.
    //     2. When the solver throws `AliasResolveRecursionLimitReached` error, it looks at `log` to create an error message.
    pub fn resolve_use(&mut self, r#use: &mut Use, log: &mut Vec<Span>) -> Result<(), ()> {
        match self.name_aliases.get(&r#use.root.def_span) {
            // r#use: `use x.y.z as w;`
            // alias: `use a.b.c as x;`
            // ->
            // `use a.b.c.y.z as w;`
            Some(alias) => {
                *r#use = Use {
                    fields: vec![
                        alias.fields.clone(),
                        r#use.fields.clone(),
                    ].concat(),
                    root: alias.root,
                    ..r#use.clone()
                };
                log.push(r#use.name_span);
                log.push(r#use.root.span);
                return Ok(());
            },
            None => {},
        }

        match self.type_aliases.get(&r#use.root.def_span) {
            Some(alias) => {
                if alias.generics.is_empty() {
                    match &alias.r#type {
                        // r#use: `use x.y.z as w;`
                        // alias: `type x = a;`
                        // ->
                        // `use a.y.z as w;`
                        Type::Identifier(alias_id) => {
                            *r#use = Use {
                                root: *alias_id,
                                ..r#use.clone()
                            };
                            log.push(r#use.name_span);
                            log.push(r#use.root.span);
                            return Ok(());
                        },
                        // r#use: `use x.y.z as w;`
                        // alias: `type x = a.b.c;`
                        // ->
                        // `use a.b.c.y.z as w;`
                        Type::Path { id: alias_id, fields: alias_fields } => {
                            *r#use = Use {
                                fields: vec![
                                    alias_fields.clone(),
                                    r#use.fields.clone(),
                                ].concat(),
                                root: *alias_id,
                                ..r#use.clone()
                            };
                            log.push(r#use.name_span);
                            log.push(r#use.root.span);
                            return Ok(());
                        },

                        // ... is this an error??
                        Type::Param { r#type, args, .. } => todo!(),

                        // error
                        Type::Tuple { .. } | Type::Func { .. } |
                        Type::Wildcard(_) | Type::Never(_) => todo!(),
                    }
                }

                // r#use: `use x.y.z as w;`
                // alias: `type x<T> = _;`
                else {
                    self.errors.push(Error {
                        kind: ErrorKind::MissingTypeArgument {
                            expected: alias.generics.len(),
                            got: 0,
                        },
                        spans: vec![
                            RenderableSpan {
                                span: r#use.root.def_span,
                                auxiliary: true,
                                note: Some(format!(
                                    "It expects {} argument{}.",
                                    alias.generics.len(),
                                    if alias.generics.len() == 1 { "" } else { "s" },
                                )),
                            },
                            RenderableSpan {
                                span: r#use.root.span,
                                auxiliary: false,
                                note: Some(String::from("It has 0 arguments.")),
                            },
                        ],
                        note: None,
                    });
                    return Err(());
                }
            },
            None => {},
        }

        if let Some(field) = r#use.fields.get(0) {
            let (field_name, field_span) = match field {
                Field::Name { name, span, .. } => (*name, *span),
                _ => unreachable!(),
            };

            match self.module_name_map.get(&r#use.root.def_span) {
                Some((_, items)) => match items.get(&field_name) {
                    // r#use: `use x.y.z as w;`
                    // `x` is a module, and `y`'s def_span is `item_span`.
                    Some((item_span, item_kind)) => {
                        *r#use = Use {
                            fields: r#use.fields[1..].to_vec(),
                            root: IdentWithOrigin {
                                id: field_name,
                                span: field_span,
                                origin: NameOrigin::Foreign {
                                    kind: *item_kind,
                                },
                                def_span: *item_span,
                            },
                            ..r#use.clone()
                        };
                        log.push(r#use.root.span);
                        return Ok(());
                    },

                    // r#use: `use x.y.z as w;`
                    // `x` is a module, but `x` doesn't have an item named `y`.
                    None => {
                        self.errors.push(Error {
                            kind: ErrorKind::UndefinedName(field_name),
                            spans: field_span.simple_error(),
                            note: Some(format!(
                                "Module `{}` doesn't have an item named `{}`.",
                                String::from_utf8_lossy(&unintern_string(r#use.root.id, &self.intermediate_dir).unwrap().unwrap()),
                                String::from_utf8_lossy(&unintern_string(field_name, &self.intermediate_dir).unwrap().unwrap()),
                            )),
                        });
                        return Err(());
                    },
                },
                None => {},
            }
        }

        Ok(())
    }

    // It resolves names in type annotations and type aliases.
    // See the comments in `resolve_use` for more information.
    pub fn resolve_type(&mut self, r#type: &mut Type, log: &mut Vec<Span>) -> Result<(), ()> {
        match r#type {
            Type::Identifier(id) => {
                match self.name_aliases.get(&id.def_span) {
                    Some(alias) => {
                        // r#type: `type x = y;`
                        // alias: `use a as y;`
                        // ->
                        // `type x = a;`
                        if alias.fields.is_empty() {
                            log.push(id.span);
                            log.push(id.def_span);
                            *r#type = Type::Identifier(alias.root);
                        }

                        // r#type: `type x = y;`
                        // alias: `use a.b as y;`
                        // ->
                        // `type x = a.b;`
                        else {
                            log.push(id.span);
                            log.push(id.def_span);
                            *r#type = Type::Path {
                                id: alias.root,
                                fields: alias.fields.clone(),
                            };
                        }

                        return Ok(());
                    },
                    None => {},
                }

                match self.type_aliases.get(&id.def_span) {
                    Some(alias) => match &alias.r#type {
                        Type::Identifier(alias_id) => {
                            // r#type: `type x = y;`
                            // alias: `type y = a;`
                            // ->
                            // `type x = a;`
                            if alias.generics.is_empty() {
                                log.push(id.span);
                                log.push(id.def_span);
                                *r#type = Type::Identifier(*alias_id);
                            }

                            // r#type: `type x = y;`
                            // alias: `type y<T> = a;`
                            // error!
                            else {
                                self.errors.push(Error {
                                    kind: ErrorKind::MissingTypeArgument {
                                        expected: alias.generics.len(),
                                        got: 0,
                                    },
                                    spans: vec![
                                        RenderableSpan {
                                            span: id.def_span,
                                            auxiliary: true,
                                            note: Some(format!(
                                                "It expects {} argument{}.",
                                                alias.generics.len(),
                                                if alias.generics.len() == 1 { "" } else { "s" },
                                            )),
                                        },
                                        RenderableSpan {
                                            span: id.span,
                                            auxiliary: false,
                                            note: Some(String::from("It has 0 arguments.")),
                                        },
                                    ],
                                    note: None,
                                });
                                return Err(());
                            }
                        },
                        Type::Path { id: alias_id, fields: alias_fields } => todo!(),
                        Type::Param { r#type: alias_t, args, .. } => todo!(),
                        _ => todo!(),
                    },
                    None => {},
                }

                Ok(())
            },
            Type::Path { id, fields } => todo!(),
            Type::Param { r#type: p_type, args, group_span } => match &**p_type {
                Type::Identifier(id) => {
                    match self.name_aliases.get(&id.def_span) {
                        Some(alias) => {
                            // r#type: `type x = y<Int>;`
                            // alias: `use a as y;`
                            // ->
                            // `type x = a<Int>;`
                            if alias.fields.is_empty() {
                                log.push(id.span);
                                log.push(id.def_span);
                                *r#type = Type::Param {
                                    r#type: Box::new(Type::Identifier(alias.root)),
                                    args: args.clone(),
                                    group_span: *group_span,
                                };
                            }

                            // r#type: `type x = y<Int>;`
                            // alias: `use a.b.c as y;`
                            // ->
                            // `type x = a.b.c<Int>;`
                            else {
                                log.push(id.span);
                                log.push(id.def_span);
                                *r#type = Type::Param {
                                    r#type: Box::new(Type::Path {
                                        id: alias.root,
                                        fields: alias.fields.clone(),
                                    }),
                                    args: args.clone(),
                                    group_span: *group_span,
                                };
                            }

                            return Ok(());
                        },
                        None => {},
                    }

                    match self.type_aliases.get(&id.def_span) {
                        Some(alias) => todo!(),
                        None => {},
                    }

                    Ok(())
                },
                Type::Path { id, fields } => todo!(),
                _ => unreachable!(),
            },
            Type::Func { r#return, args, .. } => {
                let mut has_error = false;

                if let Err(()) = self.resolve_type(r#return, log) {
                    has_error = true;
                }

                for arg in args.iter_mut() {
                    if let Err(()) = self.resolve_type(arg, log) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Type::Tuple { types, .. } => {
                let mut has_error = false;

                for r#type in types.iter_mut() {
                    if let Err(()) = self.resolve_type(r#type, log) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Type::Wildcard(_) | Type::Never(_) => Ok(()),
        }
    }

    pub fn resolve_expr(&mut self, expr: &mut Expr) -> Result<(), ()> {
        match expr {
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => Ok(()),
            Expr::Identifier(id) => {
                match self.name_aliases.get(&id.def_span) {
                    Some(alias) => todo!(),
                    None => {},
                }

                match self.type_aliases.get(&id.def_span) {
                    Some(alias) => todo!(),
                    None => {},
                }

                Ok(())
            },
            Expr::If(r#if) => match (
                self.resolve_expr(&mut r#if.cond),
                self.resolve_expr(&mut r#if.true_value),
                self.resolve_expr(&mut r#if.false_value),
            ) {
                (Ok(()), Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
            Expr::Match(r#match) => todo!(),
            Expr::Block(block) => {
                let mut has_error = false;

                for r#let in block.lets.iter_mut() {
                    if let Err(()) = self.resolve_let(r#let) {
                        has_error = true;
                    }
                }

                for assert in block.asserts.iter_mut() {
                    if let Err(()) = self.resolve_assert(assert) {
                        has_error = true;
                    }
                }

                if let Err(()) = self.resolve_expr(&mut block.value) {
                    has_error = true;
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Call { func, args } => {
                let mut has_error = false;

                if let Err(()) = self.resolve_expr(func) {
                    has_error = true;
                }

                for arg in args.iter_mut() {
                    if let Err(()) = self.resolve_expr(&mut arg.arg) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => self.resolve_expr(hs),
            Expr::InfixOp { lhs, rhs, .. } => match (
                self.resolve_expr(lhs),
                self.resolve_expr(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
            _ => panic!("TODO: {expr:?}"),
        }
    }
}
