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
use sodigy_span::{PolySpanKind, RenderableSpan, Span, SpanDeriveKind};
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
    // `type foo as y;`
    // `type foo as z;`
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

                if let Err(()) = self.resolve_type(&mut alias.r#type, &mut alias_log, 0) {
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
            }

            match path {
                Expr::Ident(id) => match self.polys.get_mut(&id.def_span) {
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
                Type::Ident(id) => match id.origin {
                    NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                        NameKind::Struct => Ok((true, id.def_span)),
                        NameKind::Enum => Ok((false, id.def_span)),
                        NameKind::Generic => Err(Error {
                            kind: ErrorKind::TooGeneralToAssociateItem,
                            spans: associated_item.type_span.simple_error(),
                            note: None,
                        }),

                        // already filtered out by hir
                        _ => unreachable!(),
                    },

                    // already filtered out by hir
                    _ => unreachable!(),
                },

                // it has to be resolved already
                Type::Path { .. } => unreachable!(),

                Type::Param { constructor, .. } => get_def_span(associated_item, constructor),
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
            if let Err(()) = self.resolve_type(&mut associated_item.r#type, &mut vec![], 0) {
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
                                    let generic_names = (0..(params + 1)).map(
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
                                                name: generic_names[i],
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
                                                type_annot: Some(Type::Ident(IdentWithOrigin {
                                                    id: generic_names[i],
                                                    span: Span::None,
                                                    def_span: Span::Poly {
                                                        name: poly_name_interned,
                                                        kind: PolySpanKind::Param(i),
                                                    },
                                                    origin: NameOrigin::Generic { index: i },
                                                })),
                                                default_value: None,
                                            }
                                        ).collect(),
                                        type_annot: Some(Type::Ident(IdentWithOrigin {
                                            id: generic_names[params],
                                            span: Span::None,
                                            def_span: Span::Poly {
                                                name: poly_name_interned,
                                                kind: PolySpanKind::Return,
                                            },
                                            origin: NameOrigin::Generic { index: params },
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
            if let Err(()) = self.resolve_type(&mut type_assertion.r#type, &mut vec![], 0) {
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
            if let Err(()) = self.resolve_type(type_annot, &mut vec![], 0) {
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

        for param in func.params.iter_mut() {
            if let Some(type_annot) = &mut param.type_annot {
                if let Err(()) = self.resolve_type(type_annot, &mut vec![], 0) {
                    has_error = true;
                }
            }
        }

        if let Some(type_annot) = &mut func.type_annot {
            if let Err(()) = self.resolve_type(type_annot, &mut vec![], 0) {
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

    pub fn resolve_struct(&mut self, r#struct: &mut Struct) -> Result<(), ()> {
        let mut has_error = false;

        for field in r#struct.fields.iter_mut() {
            if let Some(type_annot) = &mut field.type_annot {
                if let Err(()) = self.resolve_type(type_annot, &mut vec![], 0) {
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
        match self.name_aliases.get(&r#use.root.def_span) {
            // r#use: `use x.y.z as w;`
            // alias: `use a.b.c as x;`
            // ->
            // `use a.b.c.y.z as w;`
            // `a`, `b` and `c` in the new `use` statement inherit spans
            // from `x`, for better error messages.
            Some(alias) => {
                let alias_fields = alias.fields.iter().map(
                    |field| match field {
                        Field::Name { name, .. } => Field::Name {
                            name: *name,
                            name_span: r#use.root.span,
                            dot_span: r#use.root.span,
                            is_from_alias: true,
                        },
                        _ => unreachable!(),
                    }
                ).collect();
                *r#use = Use {
                    fields: vec![
                        alias_fields,
                        r#use.fields.clone(),
                    ].concat(),
                    root: IdentWithOrigin {
                        def_span: alias.root.def_span,
                        origin: alias.root.origin,
                        ..r#use.root
                    },
                    ..r#use.clone()
                };
                log.push(r#use.name_span);
                log.push(r#use.root.def_span);
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
                        Type::Ident(alias_id) => {
                            *r#use = Use {
                                root: IdentWithOrigin {
                                    def_span: alias_id.def_span,
                                    origin: alias_id.origin,
                                    ..r#use.root
                                },
                                ..r#use.clone()
                            };
                            log.push(r#use.name_span);
                            log.push(alias_id.span);
                            return Ok(());
                        },
                        // r#use: `use x.y.z as w;`
                        // alias: `type x = a.b.c;`
                        // ->
                        // `use a.b.c.y.z as w;`
                        // `a`, `b` and `c` in the new `use` statement inherit spans
                        // from `x`, for better error messages.
                        Type::Path { id: alias_id, fields: alias_fields } => {
                            let alias_fields = alias_fields.iter().map(
                                |field| match field {
                                    Field::Name { name, .. } => Field::Name {
                                        name: *name,
                                        name_span: r#use.root.span,
                                        dot_span: r#use.root.span,
                                        is_from_alias: true,
                                    },
                                    _ => unreachable!(),
                                }
                            ).collect();
                            *r#use = Use {
                                fields: vec![
                                    alias_fields,
                                    r#use.fields.clone(),
                                ].concat(),
                                root: IdentWithOrigin {
                                    def_span: alias_id.def_span,
                                    origin: alias_id.origin,
                                    ..r#use.root
                                },
                                ..r#use.clone()
                            };
                            log.push(r#use.name_span);
                            log.push(alias_id.span);
                            return Ok(());
                        },

                        // We have to convert a name alias into a type alias.
                        // `type Tuple2 = (Int, Int);`
                        // `use Tuple2 as MyTuple;`
                        // ->
                        // `type MyTuple = (Int, Int);`
                        Type::Param { .. } |
                        Type::Tuple { .. } |
                        Type::Func { .. } |
                        Type::Wildcard(_) |
                        Type::Never(_) => {
                            log.push(r#use.name_span);
                            log.push(r#use.root.span);
                            name_aliases_to_type_aliases.push((
                                r#use.name_span,
                                Alias {
                                    visibility: r#use.visibility.clone(),
                                    keyword_span: r#use.keyword_span,
                                    name: r#use.name,
                                    name_span: r#use.name_span,
                                    generics: vec![],
                                    generic_group_span: None,
                                    r#type: alias.r#type.clone(),
                                    foreign_names: alias.foreign_names.clone(),
                                },
                            ));
                        },
                    }
                }

                else {
                    // We have to convert a name alias into a type alias.
                    // r#use: `use x as w;`
                    // alias: `type x<T> = Option<T>;`
                    // ->
                    // `type w<T> = Option<T>;`
                    if r#use.fields.is_empty() {
                        log.push(r#use.name_span);
                        log.push(r#use.root.span);
                        name_aliases_to_type_aliases.push((
                            r#use.name_span,
                            Alias {
                                visibility: r#use.visibility.clone(),
                                keyword_span: r#use.keyword_span,
                                name: r#use.name,
                                name_span: r#use.name_span,
                                generics: alias.generics.iter().map(
                                    |generic| Generic {
                                        name: generic.name,
                                        name_span: r#use.root.span,
                                        // TODO: we need an extra field that it's from an alias
                                    }
                                ).collect(),
                                generic_group_span: Some(r#use.root.span),
                                r#type: alias.r#type.clone(),
                                foreign_names: alias.foreign_names.clone(),
                            },
                        ));
                    }

                    // r#use: `use x.y.z as w;`
                    // alias: `type x<T> = _;`
                    // -> error
                    else {
                        let (field_name, field_span) = match r#use.fields.get(0) {
                            Some(Field::Name { name, name_span, .. }) => (*name, *name_span),
                            _ => unreachable!(),
                        };

                        self.errors.push(Error {
                            kind: ErrorKind::UndefinedName(field_name),
                            spans: vec![
                                RenderableSpan {
                                    span: field_span,
                                    auxiliary: false,
                                    note: None,
                                },
                                RenderableSpan {
                                    span: r#use.root.span,
                                    auxiliary: true,
                                    note: Some(format!(
                                        "`{}` is just a type alias. It doesn't have any fields.",
                                        r#use.root.id.unintern_or_default(&self.intermediate_dir),
                                    )),
                                },
                            ],
                            note: None,
                        });
                        return Err(());
                    }
                }
            },
            None => {},
        }

        if let Some(field) = r#use.fields.get(0) {
            let (field_name, field_span, is_from_alias) = match field {
                Field::Name { name, name_span, is_from_alias, .. } => (*name, *name_span, *is_from_alias),
                _ => unreachable!(),
            };

            match self.item_name_map.get(&r#use.root.def_span) {
                Some((kind @ (NameKind::Module | NameKind::Enum), items)) => match items.get(&field_name) {
                    // r#use: `use x.y.z as w;`
                    // `x` is a module, and `y`'s def_span is `item_span`.
                    // or,
                    // `x` is an enum and `y` is a variant. again, `y`'s def_span is `item_span`.
                    Some((item_span, item_kind)) => {
                        *r#use = Use {
                            fields: r#use.fields[1..].to_vec(),
                            root: IdentWithOrigin {
                                id: field_name,
                                span: field_span,
                                origin: NameOrigin::Foreign { kind: *item_kind },
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
                        // al1: `use x.y.z as w;`
                        // al2: `use w as k;`
                        // Let's say `x` doesn't have an item named `y`.
                        // We have to throw UndefinedName error only once: only at al1, not at al2.
                        if !is_from_alias {
                            let error_message = match kind {
                                NameKind::Module => format!(
                                    "Module `{}` doesn't have an item named `{}`.",
                                    r#use.root.id.unintern_or_default(&self.intermediate_dir),
                                    field_name.unintern_or_default(&self.intermediate_dir),
                                ),
                                NameKind::Enum => format!(
                                    "Enum `{}` doesn't have a variant named `{}`.",
                                    r#use.root.id.unintern_or_default(&self.intermediate_dir),
                                    field_name.unintern_or_default(&self.intermediate_dir),
                                ),
                                _ => unreachable!(),
                            };

                            self.errors.push(Error {
                                kind: ErrorKind::UndefinedName(field_name),
                                spans: field_span.simple_error(),
                                note: Some(error_message),
                            });
                        }

                        return Err(());
                    },
                },
                Some((_, _)) => todo!(),
                None => {},
            }
        }

        Ok(())
    }

    // It resolves names in type annotations and type aliases.
    // See the comments in `resolve_use` for more information.
    pub fn resolve_type(
        &mut self,
        r#type: &mut Type,
        log: &mut Vec<Span>,
        recursion_depth: usize,
    ) -> Result<(), ()> {
        if recursion_depth == ALIAS_RESOLVE_RECURSION_LIMIT {
            self.errors.push(Error {
                kind: ErrorKind::AliasResolveRecursionLimitReached,
                spans: r#type.error_span_wide().simple_error(),
                note: Some(String::from("Recursion limit reached while trying to resolve this type annotation. It's likely that there's a recursive alias.")),
            });
            return Err(());
        }

        match r#type {
            Type::Ident(id) => {
                match self.name_aliases.get(&id.def_span) {
                    Some(alias) => {
                        // r#type: `type x = y;`
                        // alias: `use a as y;`
                        // ->
                        // `type x = a;`
                        if alias.fields.is_empty() {
                            log.push(id.span);
                            log.push(id.def_span);
                            *r#type = Type::Ident(IdentWithOrigin {
                                def_span: alias.root.def_span,
                                origin: alias.root.origin,
                                ..*id
                            });
                        }

                        // r#type: `type x = y;`
                        // alias: `use a.b as y;`
                        // ->
                        // `type x = a.b;`
                        else {
                            log.push(id.span);
                            log.push(id.def_span);
                            *r#type = Type::Path {
                                id: IdentWithOrigin {
                                    def_span: alias.root.def_span,
                                    origin: alias.root.origin,
                                    ..*id
                                },
                                fields: alias.fields.iter().map(
                                    |field| match field {
                                        Field::Name { name, .. } => Field::Name {
                                            name: *name,
                                            name_span: id.span,
                                            dot_span: id.span,
                                            is_from_alias: true,
                                        },
                                        _ => unreachable!(),
                                    }
                                ).collect(),
                            };
                        }

                        return Ok(());
                    },
                    None => {},
                }

                match self.type_aliases.get(&id.def_span) {
                    Some(alias) => match &alias.r#type {
                        Type::Ident(alias_id) => {
                            // r#type: `type x = y;`
                            // alias: `type y = a;`
                            // ->
                            // `type x = a;`
                            if alias.generics.is_empty() {
                                log.push(id.span);
                                log.push(id.def_span);
                                let mut alias = alias.r#type.clone();
                                alias.replace_name_and_span(id.id, id.span);
                                *r#type = alias;
                            }

                            // r#type: `type x = y;`
                            // alias: `type y<T> = a;`
                            // error!
                            // TODO: It's not an error, it's just an alias!!
                            //       But this function cannot make this alias...
                            else {
                                self.errors.push(Error {
                                    kind: ErrorKind::MissingTypeParameter {
                                        expected: alias.generics.len(),
                                        got: 0,
                                    },
                                    spans: vec![
                                        RenderableSpan {
                                            span: id.def_span,
                                            auxiliary: true,
                                            note: Some(format!(
                                                "It has {} parameter{}.",
                                                alias.generics.len(),
                                                if alias.generics.len() == 1 { "" } else { "s" },
                                            )),
                                        },
                                        RenderableSpan {
                                            span: id.span,
                                            auxiliary: false,
                                            note: Some(String::from("There are 0 arguments.")),
                                        },
                                    ],
                                    note: None,
                                });
                                return Err(());
                            }
                        },
                        Type::Path { id: alias_id, fields: alias_fields } => todo!(),
                        Type::Param { .. } => {
                            // r#type: `type x = y;`
                            // alias: `type y = Option<Int>;`
                            if alias.generics.is_empty() {
                                log.push(id.span);
                                log.push(id.def_span);
                                let mut alias = alias.r#type.clone();
                                alias.replace_name_and_span(id.id, id.span);
                                *r#type = alias;
                            }

                            // r#type: `type x = y;`
                            // alias: `type y<T> = Option<T>;`
                            // TODO: This is not an error, it's just an alias.
                            //       But this function cannot make this kinda alias.
                            else {
                                todo!()
                            }
                        },
                        Type::Tuple { types, .. } => {
                            // r#type: `type x = y;`
                            // alias: `type y = (Int, Int);`
                            if alias.generics.is_empty() {
                                log.push(id.span);
                                log.push(id.def_span);
                                // TODO: do we have to replace all the spans inside `types`?
                                let alias = Type::Tuple {
                                    types: types.clone(),
                                    group_span: id.span.derive(SpanDeriveKind::Trivial),
                                };
                                *r#type = alias;
                            }

                            else {
                                todo!()
                            }
                        },
                        Type::Func { params, r#return, purity, .. } => {
                            // r#type: `type x = y;`
                            // alias: `type y = Fn(Int) -> Int;`
                            if alias.generics.is_empty() {
                                log.push(id.span);
                                log.push(id.def_span);
                                // TODO: do we have to replace all the spans inside `params` and `r#return`?
                                let alias = Type::Func {
                                    fn_span: id.span.derive(SpanDeriveKind::Trivial),
                                    group_span: id.span.derive(SpanDeriveKind::Trivial),
                                    params: params.clone(),
                                    r#return: r#return.clone(),
                                    purity: *purity,
                                };
                                *r#type = alias;
                            }

                            else {
                                todo!()
                            }
                        },
                        _ => todo!(),
                    },
                    None => {},
                }

                Ok(())
            },
            Type::Path { id, fields } => {
                match self.name_aliases.get(&id.def_span) {
                    Some(alias) => {
                        // r#type: `type x = y.z;`
                        // alias: `use a as y;`
                        if alias.fields.is_empty() {
                            todo!()
                        }

                        // r#type: `type x = y.z;`
                        // alias: `use a.b as y;`
                        else {
                            todo!()
                        }
                    },
                    None => {},
                }

                match self.type_aliases.get(&id.def_span) {
                    Some(alias) => todo!(),
                    None => {},
                }

                match self.item_name_map.get(&id.def_span) {
                    Some((NameKind::Module, items)) => {
                        let (field_name, field_span) = (fields[0].unwrap_name(), fields[0].unwrap_name_span());

                        match items.get(&field_name) {
                            Some((item, item_kind)) => {
                                log.push(id.span);
                                log.push(id.def_span);
                                let new_id = IdentWithOrigin {
                                    id: field_name,
                                    span: field_span,
                                    origin: NameOrigin::Foreign { kind: *item_kind },
                                    def_span: *item,
                                };

                                if fields.len() == 1 {
                                    *r#type = Type::Ident(new_id);
                                    Ok(())
                                }

                                else {
                                    *r#type = Type::Path {
                                        id: new_id,
                                        fields: fields[1..].to_vec(),
                                    };
                                    self.resolve_type(r#type, log, recursion_depth + 1)
                                }
                            },
                            None => {
                                self.errors.push(Error {
                                    kind: ErrorKind::UndefinedName(field_name),
                                    spans: field_span.simple_error(),
                                    note: Some(format!(
                                        "Module `{}` doesn't have an item named `{}`.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                        field_name.unintern_or_default(&self.intermediate_dir),
                                    )),
                                });
                                Err(())
                            },
                        }
                    },
                    // r#type: `type x = Option.Some;`
                    Some((NameKind::Enum, items)) => {
                        let (field_name, field_span) = (fields[0].unwrap_name(), fields[0].unwrap_name_span());

                        match items.get(&field_name) {
                            Some(_) => {
                                self.errors.push(Error {
                                    kind: ErrorKind::EnumVariantInTypeAnnotation,
                                    spans: field_span.simple_error(),
                                    note: None,
                                });
                                Err(())
                            },
                            None => {
                                self.errors.push(Error {
                                    kind: ErrorKind::UndefinedName(field_name),
                                    spans: field_span.simple_error(),
                                    note: Some(format!(
                                        "Enum `{}` doesn't have a variant named `{}`.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                        field_name.unintern_or_default(&self.intermediate_dir),
                                    )),
                                });
                                Err(())
                            },
                        }
                    },
                    Some((_, _)) => unreachable!(),
                    None => Ok(()),
                }
            },
            Type::Param { constructor, args, group_span } => {
                for arg in args.iter_mut() {
                    self.resolve_type(arg, log, recursion_depth + 1)?;
                }

                match &**constructor {
                    Type::Ident(id) => {
                        let id = *id;

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
                                        constructor: Box::new(Type::Ident(IdentWithOrigin {
                                            def_span: alias.root.def_span,
                                            origin: alias.root.origin,
                                            ..id
                                        })),
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
                                        constructor: Box::new(Type::Path {
                                            id: IdentWithOrigin {
                                                def_span: alias.root.def_span,
                                                origin: alias.root.origin,
                                                ..id
                                            },
                                            fields: alias.fields.iter().map(
                                                |field| match field {
                                                    Field::Name { name, .. } => Field::Name {
                                                        name: *name,
                                                        name_span: id.span,
                                                        dot_span: id.span,
                                                        is_from_alias: true,
                                                    },
                                                    _ => unreachable!(),
                                                }
                                            ).collect(),
                                        }),
                                        args: args.clone(),
                                        group_span: *group_span,
                                    };
                                }
                            },
                            None => {},
                        }

                        match self.type_aliases.get(&id.def_span) {
                            Some(alias) => todo!(),
                            None => {},
                        }
                    },
                    // r#type: `type x = std.prelude.Option<Int>`
                    Type::Path { id, fields } => {
                        match self.name_aliases.get(&id.def_span) {
                            Some(alias) => todo!(),
                            None => {},
                        }

                        match self.type_aliases.get(&id.def_span) {
                            Some(alias) => todo!(),
                            None => {},
                        }

                        match self.item_name_map.get(&id.def_span) {
                            Some((NameKind::Module, items)) => {
                                let (field_name, field_span) = (fields[0].unwrap_name(), fields[0].unwrap_name_span());

                                match items.get(&field_name) {
                                    Some((item, item_kind)) => {
                                        log.push(id.span);
                                        log.push(id.def_span);
                                        let new_id = IdentWithOrigin {
                                            id: field_name,
                                            span: field_span,
                                            origin: NameOrigin::Foreign { kind: *item_kind },
                                            def_span: *item,
                                        };

                                        if fields.len() == 1 {
                                            *r#type = Type::Param {
                                                constructor: Box::new(Type::Ident(new_id)),
                                                args: args.clone(),
                                                group_span: *group_span,
                                            };
                                        }

                                        else {
                                            *r#type = Type::Param {
                                                constructor: Box::new(Type::Path {
                                                    id: new_id,
                                                    fields: fields[1..].to_vec(),
                                                }),
                                                args: args.clone(),
                                                group_span: *group_span,
                                            };
                                        }

                                        return self.resolve_type(r#type, log, recursion_depth + 1);
                                    },
                                    None => {
                                        self.errors.push(Error {
                                            kind: ErrorKind::UndefinedName(field_name),
                                            spans: field_span.simple_error(),
                                            note: Some(format!(
                                                "Module `{}` doesn't have an item named `{}`.",
                                                id.id.unintern_or_default(&self.intermediate_dir),
                                                field_name.unintern_or_default(&self.intermediate_dir),
                                            )),
                                        });
                                        return Err(());
                                    },
                                }
                            },
                            // r#type: `type x = Option.Some<Int>;`
                            Some((NameKind::Enum, items)) => {
                                let (field_name, field_span) = (fields[0].unwrap_name(), fields[0].unwrap_name_span());

                                match items.get(&field_name) {
                                    Some(_) => {
                                        self.errors.push(Error {
                                            kind: ErrorKind::EnumVariantInTypeAnnotation,
                                            spans: field_span.simple_error(),
                                            note: None,
                                        });
                                        return Err(());
                                    },
                                    None => {
                                        self.errors.push(Error {
                                            kind: ErrorKind::UndefinedName(field_name),
                                            spans: field_span.simple_error(),
                                            note: Some(format!(
                                                "Enum `{}` doesn't have a variant named `{}`.",
                                                id.id.unintern_or_default(&self.intermediate_dir),
                                                field_name.unintern_or_default(&self.intermediate_dir),
                                            )),
                                        });
                                        return Err(());
                                    },
                                }
                            },
                            Some((_, _)) => unreachable!(),
                            None => {},
                        }
                    },
                    _ => unreachable!(),
                }

                Ok(())
            },
            Type::Func { r#return, params, .. } => {
                let mut has_error = false;

                if let Err(()) = self.resolve_type(r#return, log, recursion_depth + 1) {
                    has_error = true;
                }

                for param in params.iter_mut() {
                    if let Err(()) = self.resolve_type(param, log, recursion_depth + 1) {
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
                    if let Err(()) = self.resolve_type(r#type, log, recursion_depth + 1) {
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
            Expr::Ident(id) => {
                match self.name_aliases.get(&id.def_span) {
                    Some(alias) => {
                        // expr: `Bool`
                        // alias: `use x as Bool;`
                        if alias.fields.is_empty() {
                            *id = IdentWithOrigin {
                                def_span: alias.root.def_span,
                                origin: alias.root.origin,
                                ..*id
                            };
                        }

                        // expr: `Bool`
                        // alias: `use std.Bool as Bool;`
                        else {
                            todo!()
                        }
                    },
                    None => {},
                }

                match self.type_aliases.get(&id.def_span) {
                    Some(alias) => panic!("id: {id:?}, alias: {alias:?}"),
                    None => {},
                }

                Ok(())
            },
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
            Expr::Path { lhs, fields } => {
                self.resolve_expr(lhs)?;

                match &**lhs {
                    Expr::Ident(id) => match self.item_name_map.get(&id.def_span) {
                        Some((kind @ (NameKind::Module | NameKind::Enum), items)) => {
                            let (field_name, field_span) = (fields[0].unwrap_name(), fields[0].unwrap_name_span());

                            match items.get(&field_name) {
                                Some((item, item_kind)) => {
                                    let new_root = Expr::Ident(IdentWithOrigin {
                                        id: field_name,
                                        span: field_span,
                                        origin: NameOrigin::Foreign { kind: *item_kind },
                                        def_span: *item,
                                    });

                                    if fields.len() == 1 {
                                        *expr = new_root;
                                        Ok(())
                                    }

                                    else {
                                        *expr = Expr::Path {
                                            lhs: Box::new(new_root),
                                            fields: fields[1..].to_vec(),
                                        };
                                        self.resolve_expr(expr)
                                    }
                                },
                                None => {
                                    let error_message = match kind {
                                        NameKind::Module => format!(
                                            "Module `{}` doesn't have an item named `{}`.",
                                            id.id.unintern_or_default(&self.intermediate_dir),
                                            field_name.unintern_or_default(&self.intermediate_dir),
                                        ),
                                        NameKind::Enum => format!(
                                            "Enum `{}` doesn't have a variant named `{}`.",
                                            id.id.unintern_or_default(&self.intermediate_dir),
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
                        Some((_, _)) => todo!(),
                        None => Ok(()),
                    },
                    Expr::Path { .. } => todo!(),

                    // `(1 + 2).a.b` -> `a` and `b` are fields
                    _ => Ok(()),
                }
            },
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => self.resolve_expr(hs),
            Expr::FieldModifier { lhs, rhs, .. } |
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
            PatternKind::Ident { .. } |
            PatternKind::Number { .. } |
            PatternKind::String { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } |
            PatternKind::Wildcard(_) => Ok(()),
            PatternKind::Path(_) => todo!(),
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
            _ => panic!("TODO: {kind:?}"),
        }
    }
}
