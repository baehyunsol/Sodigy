use crate::{Assert, Func, Let, Type};
use sodigy_error::{Error, Warning};
use sodigy_hir::{self as hir, FuncArgDef, StructField};
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,
    pub func_shapes: HashMap<Span, Vec<FuncArgDef<()>>>,
    pub struct_shapes: HashMap<Span, Vec<StructField<()>>>,
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub asserts: Vec<Assert>,

    // It's `def_span -> type_annotation` map.
    // It has type information of *every* name in the code.
    // If you query a def_span of a function, it'll give you something like `Fn(Int, Int) -> Int`.
    //
    // If first collects the type annotations, then the type-infer engine will infer the
    // missing type annotations.
    // Then the type-checker will check if all the annotations are correct.
    pub types: HashMap<Span, Type>,

    // We need this when we create error messages.
    // This is really expensive to initialize, so think twice before you init this.
    pub span_string_map: HashMap<Span, String>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_hir_session(hir_session: &hir::Session) -> Session {
        Session {
            intermediate_dir: hir_session.intermediate_dir.clone(),
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
            types: HashMap::new(),
            span_string_map: HashMap::new(),
            errors: hir_session.errors.clone(),
            warnings: hir_session.warnings.clone(),
        }
    }

    pub fn init_span_string_map(&mut self) {
        // TODO: as of now, there're no structs and enums in MirSession, so the error messages don't need this function.
        //       I'll implement this function when I need this
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
