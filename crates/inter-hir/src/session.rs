use sodigy_error::{Error, Warning};
use sodigy_hir::{FuncArgDef, GenericDef, StructField};
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,

    pub func_shapes: HashMap<Span, (Vec<FuncArgDef<()>>, Vec<GenericDef>)>,
    pub struct_shapes: HashMap<Span, (Vec<StructField<()>>, Vec<GenericDef>)>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Session {
        Session {
            intermediate_dir: intermediate_dir.to_string(),
            func_shapes: HashMap::new(),
            struct_shapes: HashMap::new(),
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
