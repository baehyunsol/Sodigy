use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    Alias,
    Assert,
    AssociatedItem,
    AssociatedItemKind,
    Expr,
    ExprOrString,
    Func,
    FuncOrigin,
    FuncParam,
    FuncShape,
    Generic,
    Let,
    Path,
    Pattern,
    PatternKind,
    Poly,
    Session as HirSession,
    Struct,
    StructField,
    StructShape,
    Type,
    Use,
    Visibility,
};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::{PolySpanKind, RenderableSpan, Span};
use sodigy_string::intern_string;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

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
        for (def_span, func_shape) in hir_session.funcs.iter().map(
            |func| (
                func.name_span,
                FuncShape {
                    params: func.params.iter().map(
                        |param| FuncParam {
                            name: param.name,
                            name_span: param.name_span,
                            type_annot: None,
                            default_value: param.default_value,
                        }
                    ).collect(),
                    generics: func.generics.clone(),
                },
            )
        ) {
            self.func_shapes.insert(def_span, func_shape);
        }

        for (def_span, struct_shape) in hir_session.structs.iter().map(
            |r#struct| (
                r#struct.name_span,
                StructShape {
                    name: r#struct.name,
                    fields: r#struct.fields.iter().map(
                        |field| StructField {
                            name: field.name,
                            name_span: field.name_span,
                            type_annot: None,
                            default_value: field.default_value,
                        }
                    ).collect(),
                    generics: r#struct.generics.clone(),
                    associated_funcs: HashMap::new(),
                    associated_lets: HashMap::new(),
                },
            )
        ) {
            self.struct_shapes.insert(def_span, struct_shape);
        }

        let mut children = HashMap::new();

        for (name, span, kind) in hir_session.iter_item_names() {
            children.insert(name, (span, kind));
        }

        self.item_name_map.insert(
            module_span,
            (
                NameKind::Module,
                children,
            ),
        );

        for r#enum in hir_session.enums.into_iter() {
            let mut variants = HashMap::new();

            for variant in r#enum.variants.iter() {
                variants.insert(
                    variant.name,
                    (
                        variant.name_span,
                        NameKind::EnumVariant { parent: r#enum.name_span },
                    ),
                );
            }

            self.item_name_map.insert(
                r#enum.name_span,
                (
                    NameKind::Enum,
                    variants,
                ),
            );
        }

        for (name, span) in hir_session.lang_items.into_iter() {
            self.lang_items.insert(name, span);
        }

        for r#use in hir_session.uses.drain(..) {
            self.name_aliases.insert(r#use.name_span, r#use);
        }

        for alias in hir_session.aliases.drain(..) {
            self.type_aliases.insert(alias.name_span, alias);
        }

        self.polys.extend(hir_session.polys.drain());
        self.poly_impls.extend(hir_session.poly_impls.drain(..));
        self.associated_items.extend(hir_session.associated_items.drain(..));
    }

    // Aliases might be nested. e.g.
    // `type x = foo;`
    // `use x as y;`
    // `use y as z;`
    //
    // Then, we have to resolve the above aliases to
    // `type x = foo;`
    // `use foo as y;`
    // `use foo as z;`
    //
    // We have to do this before resolving aliases in expressions and type annotations.
    // We have to do this globally.
    // Also, there can be an infinite loop, so we have to set some kinda recursion limit.
    pub fn resolve_alias(&mut self) -> Result<(), ()> {
        let mut nested_name_aliases = HashMap::new();
        let mut nested_type_aliases = HashMap::new();
        let mut name_aliases_to_type_aliases = vec![];
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

                if let Err(()) = self.resolve_use(&mut r#use, &mut name_aliases_to_type_aliases, &mut alias_log) {
                    has_error = true;
                }

                if !alias_log.is_empty() {
                    // `use x.a.b as y;`
                    // `use y.c.d as x;`
                    // -> When you resolve this `n` times, the length of the
                    //    field will be `2^n`.
                    if r#use.path.fields.len() > 1024 {
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

            else if !nested_name_aliases.is_empty() || !nested_type_aliases.is_empty() || !name_aliases_to_type_aliases.is_empty() {
                for (name_span, r#use) in nested_name_aliases.drain() {
                    self.name_aliases.insert(name_span, r#use);
                }

                for (name_span, alias) in nested_type_aliases.drain() {
                    let old_alias = self.type_aliases.get_mut(&name_span).unwrap();
                    old_alias.r#type = alias;
                }

                for (name_span, type_alias) in name_aliases_to_type_aliases.drain(..) {
                    self.name_aliases.remove(&name_span);
                    self.type_aliases.insert(name_span, type_alias);
                }
            }

            else {
                break;
            }
        }

        Ok(())
    }

    pub fn resolve_poly(&mut self) -> Result<(), ()> {
        let mut has_error = false;

        for (mut path, impl_span) in self.poly_impls.clone().into_iter() {
            if let Err(()) = self.resolve_expr(&mut path) {
                has_error = true;
                continue;
            }
        
            if let Err(()) = self.check_expr(&path) {
                has_error = true;
                continue;
            }

            match path {
                Expr::Path(Path { id, fields }) if fields.is_empty() => match self.polys.get_mut(&id.def_span) {
                    Some(poly) => {
                        poly.impls.push(impl_span);
                    },
                    None => {
                        let is_func = match id.origin {
                            NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => kind == NameKind::Func,
                            _ => false,
                        };

                        self.errors.push(Error {
                            kind: ErrorKind::NotPolyGeneric { id: Some(id) },
                            spans: vec![
                                RenderableSpan {
                                    span: id.span,
                                    auxiliary: false,
                                    note: Some(String::from("This is not a poly generic function.")),
                                },
                                RenderableSpan {
                                    span: id.def_span,
                                    auxiliary: true,
                                    note: Some(format!(
                                        "`{}` is defined here.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                    )),
                                },
                            ],
                            note: Some(
                                if is_func {
                                    format!(
                                        "Use `#[poly]` to make `{}` a poly generic function.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                    )
                                } else {
                                    format!(
                                        "`{}` is not even a function. Only a function can be a poly generic function.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                    )
                                }
                            ),
                        });
                        has_error = true;
                    },
                },
                _ => {
                    self.errors.push(Error {
                        kind: ErrorKind::NotPolyGeneric { id: None },
                        spans: vec![
                            RenderableSpan {
                                span: path.error_span_wide(),
                                auxiliary: false,
                                note: Some(String::from("This is not a poly generic function.")),
                            },
                        ],
                        note: Some(String::from("Only a function can be a poly generic.")),
                    });
                    has_error = true;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_associated_items(&mut self) -> Result<(), ()> {
        fn get_def_span(associated_item: &AssociatedItem, r#type: &Type) -> Result<(bool, Span), Error> {
            match r#type {
                Type::Path(path) | Type::Param { constructor: path, .. } => {
                    match path.id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                            NameKind::Struct => Ok((true, path.id.def_span)),
                            NameKind::Enum => Ok((false, path.id.def_span)),
                            NameKind::GenericParam => Err(Error {
                                kind: ErrorKind::TooGeneralToAssociateItem,
                                spans: associated_item.type_span.simple_error(),
                                note: None,
                            }),

                            // already filtered out by `check_type_annotation`
                            _ => unreachable!(),
                        },

                        // already filtered out by `check_type_annotation`
                        _ => unreachable!(),
                    }
                },
                Type::Tuple { .. } => todo!(),  // what's def_span of tuple? maybe use lang_item?
                Type::Func { .. } | Type::Never(_) => Err(Error {
                    kind: ErrorKind::CannotAssociateItem,
                    spans: associated_item.type_span.simple_error(),
                    note: None,
                }),
                Type::Wildcard(_) => Err(Error {
                    kind: ErrorKind::TooGeneralToAssociateItem,
                    spans: associated_item.type_span.simple_error(),
                    note: None,
                }),
            }
        }

        let mut has_error = false;
        let mut associated_items = self.associated_items.drain(..).collect::<Vec<_>>();

        'associated_items: for associated_item in associated_items.iter_mut() {
            if let Err(()) = self.resolve_type(&mut associated_item.r#type, &mut vec![]) {
                has_error = true;
                continue;
            }

            else if let Err(()) = self.check_type_annotation(&associated_item.r#type) {
                has_error = true;
                continue;
            }

            match get_def_span(&associated_item, &associated_item.r#type) {
                Ok((is_struct, def_span)) => {
                    if is_struct {
                        let struct_shape = self.struct_shapes.get_mut(&def_span).unwrap();

                        for (associated_item_kind, params, is_pure, name, name_span) in struct_shape.fields.iter().map(
                            |field| (AssociatedItemKind::Field, None, None, field.name, field.name_span)
                        ).chain(struct_shape.associated_funcs.iter().map(
                            // for error messages, `spans[0]` is enough
                            |(name, (params, is_pure, spans))| (AssociatedItemKind::Func, Some(*params), Some(*is_pure), *name, spans[0])
                        )).chain(struct_shape.associated_lets.iter().map(
                            |(name, name_span)| (AssociatedItemKind::Let, None, None, *name, *name_span)
                        )) {
                            if name == associated_item.name {
                                let error = match (associated_item_kind, associated_item.kind) {
                                    (AssociatedItemKind::Field | AssociatedItemKind::Let, AssociatedItemKind::Func) => todo!(),  // err
                                    (_, AssociatedItemKind::Let) => todo!(),  // err
                                    (AssociatedItemKind::Func, AssociatedItemKind::Func) => {
                                        if associated_item.params == params && associated_item.is_pure == is_pure {
                                            // okay
                                            continue;
                                        }

                                        else {
                                            todo!()  // err
                                        }
                                    },
                                    (_, AssociatedItemKind::Field) => unreachable!(),
                                    (AssociatedItemKind::Variant, _) | (_, AssociatedItemKind::Variant) => unreachable!(),
                                };

                                self.errors.push(error);
                                has_error = true;
                                continue 'associated_items;
                            }
                        }

                        if let AssociatedItemKind::Func = associated_item.kind {
                            let params = associated_item.params.unwrap();
                            let is_pure = associated_item.is_pure.unwrap();

                            match struct_shape.associated_funcs.entry(associated_item.name) {
                                Entry::Occupied(mut e) => {
                                    e.get_mut().2.push(associated_item.name_span);
                                },
                                Entry::Vacant(e) => {
                                    e.insert((params, is_pure, vec![associated_item.name_span]));
                                },
                            }

                            let poly_name = format!(
                                "associated_func::{}::{}::{params}",
                                associated_item.name.unintern_or_default(&self.intermediate_dir),
                                if is_pure { "pure" } else { "impure" },
                            );
                            let poly_name_interned = intern_string(poly_name.as_bytes(), &self.intermediate_dir).unwrap();
                            let poly_span: Span = Span::Poly {
                                name: poly_name_interned,
                                kind: PolySpanKind::Name,
                            };

                            match self.new_polys.entry(poly_span) {
                                Entry::Occupied(mut e) => {
                                    e.get_mut().impls.push(associated_item.name_span);
                                },
                                Entry::Vacant(e) => {
                                    let generic_params = (0..(params + 1)).map(
                                        |i| intern_string(
                                            if i != params {
                                                format!("T{i}")
                                            } else {
                                                String::from("V")
                                            }.as_bytes(),
                                            &self.intermediate_dir,
                                        ).unwrap()
                                    ).collect::<Vec<_>>();
                                    let param_names = (0..params).map(
                                        |i| intern_string(format!("p{i}").as_bytes(), &self.intermediate_dir).unwrap()
                                    ).collect::<Vec<_>>();

                                    e.insert(Poly {
                                        decorator_span: Span::None,
                                        name: poly_name_interned,
                                        name_span: poly_span,
                                        has_default_impl: false,
                                        impls: vec![associated_item.name_span],
                                    });

                                    // push `#[poly] fn @associated_func_unwrap_1<T1, T2>(x: T1) -> T2;` to the session.
                                    self.new_funcs.push(Func {
                                        is_pure,
                                        impure_keyword_span: None,

                                        // TODO: I'm not sure whether it should be private/public
                                        //       I'll know that when I implement the visibility checker.
                                        visibility: Visibility::private(),

                                        keyword_span: Span::None,
                                        name: poly_name_interned,
                                        name_span: poly_span,
                                        generics: (0..(params + 1)).map(
                                            |i| Generic {
                                                name: generic_params[i],
                                                name_span: Span::Poly {
                                                    name: poly_name_interned,
                                                    kind: if i == params {
                                                        PolySpanKind::Return
                                                    } else {
                                                        PolySpanKind::Param(i)
                                                    },
                                                },
                                            },
                                        ).collect(),
                                        params: (0..params).map(
                                            |i| FuncParam {
                                                name: param_names[i],
                                                name_span: Span::None,
                                                type_annot: Some(Type::Path(Path {
                                                    id: IdentWithOrigin {
                                                        id: generic_params[i],
                                                        span: Span::None,
                                                        def_span: Span::Poly {
                                                            name: poly_name_interned,
                                                            kind: PolySpanKind::Param(i),
                                                        },
                                                        origin: NameOrigin::GenericParam { index: i },
                                                    },
                                                    fields: vec![],
                                                })),
                                                default_value: None,
                                            }
                                        ).collect(),
                                        type_annot: Some(Type::Path(Path {
                                            id: IdentWithOrigin {
                                                id: generic_params[params],
                                                span: Span::None,
                                                def_span: Span::Poly {
                                                    name: poly_name_interned,
                                                    kind: PolySpanKind::Return,
                                                },
                                                origin: NameOrigin::GenericParam { index: params },
                                            },
                                            fields: vec![],
                                        })),
                                        value: Expr::dummy(),
                                        origin: FuncOrigin::AssociatedFunc,
                                        built_in: false,
                                        foreign_names: HashMap::new(),
                                        use_counts: HashMap::new(),
                                    });
                                },
                            }
                        }

                        else {
                            struct_shape.associated_lets.insert(associated_item.name, associated_item.name_span);
                        }
                    }

                    else {
                        todo!()
                    }
                },
                Err(e) => {
                    self.errors.push(e);
                    has_error = true;
                    continue;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
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

        for r#struct in hir_session.structs.iter_mut() {
            if let Err(()) = self.resolve_struct(r#struct) {
                has_error = true;
            }
        }

        // TODO: enums

        for assert in hir_session.asserts.iter_mut() {
            if let Err(()) = self.resolve_assert(assert) {
                has_error = true;
            }
        }

        for type_assertion in hir_session.type_assertions.iter_mut() {
            if let Err(()) = self.resolve_type(&mut type_assertion.r#type, &mut vec![]) {
                has_error = true;
            }

            else if let Err(()) = self.check_type_annotation(&type_assertion.r#type) {
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

        if let Some(type_annot) = &mut r#let.type_annot {
            if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                has_error = true;
            }

            else if let Err(()) = self.check_type_annotation(&type_annot) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut r#let.value) {
            has_error = true;
        }

        else if let Err(()) = self.check_expr(&r#let.value) {
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

        for param in func.params.iter_mut() {
            if let Some(type_annot) = &mut param.type_annot {
                if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                    has_error = true;
                }

                else if let Err(()) = self.check_type_annotation(type_annot) {
                    has_error = true;
                }
            }
        }

        if let Some(type_annot) = &mut func.type_annot {
            if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                has_error = true;
            }

            else if let Err(()) = self.check_type_annotation(type_annot) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut func.value) {
            has_error = true;
        }

        else if let Err(()) = self.check_expr(&func.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_struct(&mut self, r#struct: &mut Struct) -> Result<(), ()> {
        let mut has_error = false;

        for field in r#struct.fields.iter_mut() {
            if let Some(type_annot) = &mut field.type_annot {
                if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                    has_error = true;
                }

                else if let Err(()) = self.check_type_annotation(type_annot) {
                    has_error = true;
                }
            }
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

            else if let Err(()) = self.check_expr(note) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut assert.value) {
            has_error = true;
        }

        else if let Err(()) = self.check_expr(&assert.value) {
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
    // replaces the alias with `use y as w;`.
    //
    // There may be multiple levels of aliases in `use`. This function only resolves
    // one level of alias. `resolve_alias` will call this function multiple times
    // until all the aliases are resolved (or an `AliasResolveRecursionLimitReached` error).
    //
    // `log` does 2 things:
    //     1. It tells whether the function has resolved something. If `log` is not empty, something has happened.
    //     2. When the solver throws `AliasResolveRecursionLimitReached` error, it looks at `log` to create an error message.
    pub fn resolve_use(
        &mut self,
        r#use: &mut Use,
        name_aliases_to_type_aliases: &mut Vec<(Span, Alias)>,
        log: &mut Vec<Span>,
    ) -> Result<(), ()> {
        // TODO: If there are `use x as w;` and `type x<T> = Option<T>;`,
        //       we want to convert the use statement to a type alias:
        //       `type w<T> = Option<T>;`.
        //       Also, if there are `use x.y.z as w;` and `type x<T> = ...;`,
        //       it's an error because `x.y.z` doesn't make sense.
        //       But the problem is that `resolve_path` can't do anything with
        //       generic parameters in type aliases.
        self.resolve_path(&mut r#use.path, log)?;
        todo!()
    }

    // If there are complex aliases (alias of alias of alias of ...), this function
    // doesn't fully solve the path.
    // `resolve_alias` solves this issue by calling `resolve_use` and `resolve_type` over and
    // over until there's no more paths to solve. `resolve_alias` tracks whether it founds new
    // aliases or not with `log` parameter.
    //
    // The other resolvers (`resolve_expr`, `resolve_pattern`, ...) don't have such problem because
    // `resolve_alias` removes all the complex aliases.
    pub fn resolve_path(
        &mut self,
        path: &mut Path,
        log: &mut Vec<Span>,
    ) -> Result<(), ()> {
        // We have path `a.b.c`, and there's an alias `use x.y.z as a;`.
        // The path has to be resolved to `x.y.z.b.c`.
        // The resolved path keeps span and id of `a`, for better error messages.
        match self.name_aliases.get(&path.id.def_span) {
            Some(alias) => {
                log.push(path.id.span);
                log.push(path.id.def_span);
                path.id = IdentWithOrigin {
                    def_span: alias.path.id.def_span,
                    origin: alias.path.id.origin,
                    ..path.id
                };
                let alias_fields = alias.path.fields.iter().map(
                    |field| match field {
                        Field::Name { name, .. } => Field::Name {
                            name: *name,
                            name_span: path.id.span,
                            dot_span: path.id.span,
                            is_from_alias: true,
                        },
                        _ => unreachable!(),
                    }
                ).collect::<Vec<_>>();
                path.fields = vec![
                    alias_fields,
                    path.fields.to_vec(),
                ].concat();
                return Ok(());
            },
            None => {},
        }

        // What if it's a type alias with generic parameters?
        match self.type_aliases.get(&path.id.def_span) {
            Some(alias) => todo!(),
            None => {},
        }

        if let Some(field) = path.fields.get(0) {
            match self.item_name_map.get(&path.id.def_span) {
                Some((kind @ (NameKind::Module | NameKind::Enum), items)) => {
                    let (field_name, field_span) = (field.unwrap_name(), field.unwrap_name_span());

                    match items.get(&field_name) {
                        Some((item, item_kind)) => {
                            log.push(path.id.span);
                            log.push(path.id.def_span);
                            let new_id = IdentWithOrigin {
                                id: field_name,
                                span: field_span,
                                origin: NameOrigin::Foreign { kind: *item_kind },
                                def_span: *item,
                            };
                            path.id = new_id;

                            if path.fields.len() == 1 {
                                path.fields = vec![];
                                Ok(())
                            }

                            else {
                                path.fields = path.fields[1..].to_vec();
                                self.resolve_path(path, log)
                            }
                        },
                        None => {
                            let error_message = match kind {
                                NameKind::Module => format!(
                                    "Module `{}` doesn't have an item named `{}`.",
                                    path.id.id.unintern_or_default(&self.intermediate_dir),
                                    field_name.unintern_or_default(&self.intermediate_dir),
                                ),
                                NameKind::Enum => format!(
                                    "Enum `{}` doesn't have a variant named `{}`.",
                                    path.id.id.unintern_or_default(&self.intermediate_dir),
                                    field_name.unintern_or_default(&self.intermediate_dir),
                                ),
                                _ => unreachable!(),
                            };
                            self.errors.push(Error {
                                kind: ErrorKind::UndefinedName(field_name),
                                spans: field_span.simple_error(),
                                note: Some(error_message),
                            });
                            Err(())
                        },
                    }
                },
                Some((_, _)) => unreachable!(),
                None => Ok(()),
            }
        }

        else {
            Ok(())
        }
    }

    // It resolves names in type annotations and type aliases.
    // See the comments in `resolve_use` for more information.
    pub fn resolve_type(
        &mut self,
        r#type: &mut Type,
        log: &mut Vec<Span>,
    ) -> Result<(), ()> {
        match r#type {
            Type::Path(path) => {
                self.resolve_path(path, log)?;
                todo!()
            },
            Type::Param { constructor, args, group_span } => todo!(),
            Type::Func { fn_constructor, params, r#return, .. } => {
                let mut has_error = false;

                if let Err(()) = self.resolve_path(fn_constructor, log) {
                    has_error = true;
                }

                if let Err(()) = self.resolve_type(r#return, log) {
                    has_error = true;
                }

                for param in params.iter_mut() {
                    if let Err(()) = self.resolve_type(param, log) {
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
            Expr::Path(p) => {
                self.resolve_path(p, &mut vec![])?;

                if p.fields.is_empty() {
                    Ok(())
                }

                else {
                    *expr = Expr::Field {
                        lhs: Box::new(Expr::Path(Path {
                            id: p.id,
                            fields: vec![],
                        })),
                        fields: p.fields.to_vec(),
                    };
                    Ok(())
                }
            },
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => Ok(()),
            Expr::If(r#if) => match (
                self.resolve_expr(&mut r#if.cond),
                self.resolve_expr(&mut r#if.true_value),
                self.resolve_expr(&mut r#if.false_value),
            ) {
                (Ok(()), Ok(()), Ok(())) => {
                    if let Some(pattern) = &mut r#if.pattern {
                        self.resolve_pattern(pattern)
                    }

                    else {
                        Ok(())
                    }
                },
                _ => Err(()),
            },
            Expr::Match(r#match) => {
                let mut has_error = false;

                if let Err(()) = self.resolve_expr(&mut r#match.scrutinee) {
                    has_error = true;
                }

                for arm in r#match.arms.iter_mut() {
                    if let Err(()) = self.resolve_pattern(&mut arm.pattern) {
                        has_error = true;
                    }

                    if let Some(guard) = &mut arm.guard {
                        if let Err(()) = self.resolve_expr(guard) {
                            has_error = true;
                        }
                    }

                    if let Err(()) = self.resolve_expr(&mut arm.value) {
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
            Expr::Call { func, args, .. } => {
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
            Expr::FormattedString { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let ExprOrString::Expr(e) = element {
                        if let Err(()) = self.resolve_expr(e) {
                            has_error = true;
                        }
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Tuple { elements, .. } |
            Expr::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_expr(element) {
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
            Expr::StructInit { r#struct, fields, .. } => {
                let mut has_error = self.resolve_expr(r#struct).is_err();

                for field in fields.iter_mut() {
                    if let Err(()) = self.resolve_expr(&mut field.value) {
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
            Expr::Field { lhs, .. } => self.resolve_expr(lhs),
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => self.resolve_expr(hs),
            Expr::FieldUpdate { lhs, rhs, .. } |
            Expr::InfixOp { lhs, rhs, .. } => match (
                self.resolve_expr(lhs),
                self.resolve_expr(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
        }
    }

    pub fn resolve_pattern(&mut self, pattern: &mut Pattern) -> Result<(), ()> {
        let mut has_error = false;

        if let Err(()) = self.resolve_pattern_kind(&mut pattern.kind) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_pattern_kind(&mut self, kind: &mut PatternKind) -> Result<(), ()> {
        match kind {
            PatternKind::Path(path) => todo!(),
            PatternKind::NameBinding { .. } |
            PatternKind::Number { .. } |
            PatternKind::String { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } |
            PatternKind::Wildcard(_) => Ok(()),
            PatternKind::Struct { r#struct, fields, .. } => {
                let mut has_error = self.resolve_path(r#struct, &mut vec![]).is_err();

                for field in fields.iter_mut() {
                    if let Err(()) = self.resolve_pattern(&mut field.pattern) {
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
            PatternKind::TupleStruct { r#struct, elements, .. } => {
                let mut has_error = self.resolve_path(r#struct, &mut vec![]).is_err();

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_pattern(element) {
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
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_pattern(element) {
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
            PatternKind::Range { lhs, rhs, .. } => {
                let mut has_error = false;

                if let Some(lhs) = lhs {
                    if let Err(()) = self.resolve_pattern(lhs) {
                        has_error = true;
                    }
                }

                if let Some(rhs) = rhs {
                    if let Err(()) = self.resolve_pattern(rhs) {
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
            PatternKind::Or { lhs, rhs, .. } => match (
                self.resolve_pattern(lhs),
                self.resolve_pattern(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
        }
    }

    // Some names (e.g. `NameKind::Let`) are not a valid type annotation.
    pub fn check_type_annotation(&mut self, r#type: &Type) -> Result<(), ()> {
        todo!()
    }

    // Some names (e.g. `NameKind::Enum`) are not a valid expr.
    pub fn check_expr(&mut self, expr: &Expr) -> Result<(), ()> {
        todo!()
    }
}
