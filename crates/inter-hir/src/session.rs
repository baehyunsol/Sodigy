use sodigy_error::{Error, Warning};
use sodigy_hir::{Alias, Expr, FuncParam, Generic, Poly, StructField, Use};
use sodigy_name_analysis::NameKind;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,

    // of all hir files
    pub func_shapes: HashMap<Span, (Vec<FuncParam>, Vec<Generic>)>,
    pub struct_shapes: HashMap<Span, (Vec<StructField>, Vec<Generic>)>,
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

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Session {
        Session {
            intermediate_dir: intermediate_dir.to_string(),
            func_shapes: HashMap::new(),
            struct_shapes: HashMap::new(),
            name_aliases: HashMap::new(),
            type_aliases: HashMap::new(),
            item_name_map: HashMap::new(),
            lang_items: HashMap::new(),
            polys: HashMap::new(),
            poly_impls: vec![],
            errors: vec![],
            warnings: vec![],
        }
    }
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn get_warnings(&self) -> &[Warning] {
        &self.warnings
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
