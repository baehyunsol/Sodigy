use sodigy_error::{Error, Warning};
use sodigy_hir::{
    Alias,
    AssociatedItem,
    Expr,
    Func,
    FuncParam,
    FuncShape,
    Poly,
    StructField,
    StructShape,
    Use,
};
use sodigy_name_analysis::NameKind;
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,

    // of all hir files
    pub func_shapes: HashMap<Span, FuncShape>,
    pub struct_shapes: HashMap<Span, StructShape>,
    pub name_aliases: HashMap<Span, Use>,
    pub type_aliases: HashMap<Span, Alias>,

    // DefSpan of a module `foo` points to `foo` in `mod foo;`.
    // If it's the root module (lib) or std, it uses a special span `Span::Lib` or `Span::Std`.
    //
    // Let's say function `y` and `z` are defined in module `x`, so we can access `x.y` and `x.z`.
    // Then this map has an entry for `x`, which looks like
    // `item_name_map[x_span] = (NameKind::Module, { "y": (y_span, NameKind::Func), "z": (z_span, NameKind::Func) })`.
    // Later, when it finds `x.y` in the code, it'll try to replace `x.y` with `y_span` using this map.
    //
    // Let's say enum `Foo` has variants `X` and `Y`.
    // Then this map has an entry for `Foo`, which looks like
    // `item_name_map[Foo_span] = (NameKind::Enum, { "X": (X_span, NameKind::EnumVariant), "Y": (Y_span, NameKind::EnumVariant) })`
    pub item_name_map: HashMap<Span, (NameKind, HashMap<InternedString, (Span, NameKind)>)>,

    // For example, you can get def_span of `Int` from this map by querying `lang_items.get("type.Int")`.
    pub lang_items: HashMap<String, Span>,

    // It collects the polys from each module. After it ingested all the modules,
    // it resolves paths in `poly_impls` and fills `.impls` fields in polys.
    pub polys: HashMap<Span, Poly>,
    pub poly_impls: Vec<(Expr, Span)>,

    // Inter-hir may create new functions and poly-generics while resolving associated items.
    pub new_funcs: Vec<Func>,
    pub new_polys: HashMap<Span, Poly>,

    pub associated_items: Vec<AssociatedItem>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Session {
        let mut name_aliases = HashMap::new();

        // Per-file hir can use prelude names without knowing the defspan of the names.
        // They use special span `Span::Prelude(name)`. Inter-hir will find such spans and
        // replace them with the actual def_span.
        for prelude in sodigy_hir::PRELUDES {
            let name_alias = sodigy_hir::use_prelude(intern_string(prelude, intermediate_dir).unwrap());
            name_aliases.insert(name_alias.name_span, name_alias);
        }

        Session {
            intermediate_dir: intermediate_dir.to_string(),
            func_shapes: HashMap::new(),
            struct_shapes: HashMap::new(),
            name_aliases,
            type_aliases: HashMap::new(),
            item_name_map: HashMap::new(),
            lang_items: HashMap::new(),
            polys: HashMap::new(),
            poly_impls: vec![],
            new_funcs: vec![],
            new_polys: HashMap::new(),
            associated_items: vec![],
            errors: vec![],
            warnings: vec![],
        }
    }

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
}
