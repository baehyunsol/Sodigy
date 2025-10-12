use crate::{Assert, Func, Let, Type};
use sodigy_error::Error;
use sodigy_hir::{self as hir, FuncArgDef, StructField};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub intern_str_map_dir: String,
    pub intern_num_map_dir: String,
    pub func_shapes: HashMap<Span, Vec<FuncArgDef<()>>>,
    pub struct_shapes: HashMap<Span, Vec<StructField<()>>>,
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub asserts: Vec<Assert>,
    pub errors: Vec<Error>,
}

impl Session {
    pub fn from_hir_session(hir_session: &hir::Session) -> Session {
        Session {
            intern_str_map_dir: hir_session.intern_str_map_dir.clone(),
            intern_num_map_dir: hir_session.intern_num_map_dir.clone(),
            func_shapes: hir_session.funcs.iter().map(
                |func| (
                    func.name_span,
                    func.args.iter().map(
                        |arg| FuncArgDef {
                            name: arg.name,
                            name_span: arg.name_span,
                            r#type: None,
                            default_value: arg.default_value,
                        }
                    ).collect(),
                )
            ).collect(),
            struct_shapes: hir_session.structs.iter().map(
                |r#struct| (
                    r#struct.name_span,
                    r#struct.fields.iter().map(
                        |field| StructField {
                            name: field.name,
                            name_span: field.name_span,
                            r#type: None,
                            default_value: field.default_value,
                        }
                    ).collect(),
                )
            ).collect(),
            lets: vec![],
            funcs: vec![],
            asserts: vec![],
            errors: vec![],
        }
    }
}
