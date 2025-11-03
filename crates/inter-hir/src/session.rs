use sodigy_error::{Error, Warning};
use sodigy_hir::{Alias, FuncArgDef, GenericDef, StructField, Use};
use sodigy_name_analysis::NameKind;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,

    // of all hir files
    pub func_shapes: HashMap<Span, (Vec<FuncArgDef<()>>, Vec<GenericDef>)>,
    pub struct_shapes: HashMap<Span, (Vec<StructField<()>>, Vec<GenericDef>)>,

    // of the current hir file
    pub name_aliases: HashMap<Span, Use>,
    pub type_aliases: HashMap<Span, Alias>,

    // DefSpan of a module `foo` points to `foo` in `mod foo`.
    // If it's the root module (lib), it uses a special span `Span::Lib`.
    //
    // Let's say function `y` and `z` are defined in module `x`, so we can access `x.y` and `x.z`.
    // Then this map has an entry for `x`, which looks like
    // `module_name_map[x_span] = (x_span, NameKind::Module, { "y": y_span, "z": z_span })`.
    // Later, when it finds `x.y` in the code, it'll try to replace `x.y` with `y_span` using this map.
    // It's collecting `NameKind::Module` because the map can later be used to solve enum variants.
    pub module_name_map: HashMap<Span, (Span, NameKind, HashMap<InternedString, Span>)>,

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
            module_name_map: HashMap::new(),
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
