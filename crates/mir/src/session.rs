use crate::{Assert, Func, Let, Type};
use sodigy_error::{Error, Warning};
use sodigy_hir::{self as hir, FuncArgDef, GenericDef, StructField};
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

mod span_string_map;

pub struct Session {
    pub intermediate_dir: String,
    pub func_shapes: HashMap<Span, (Vec<FuncArgDef<()>>, Vec<GenericDef>)>,
    pub struct_shapes: HashMap<Span, (Vec<StructField<()>>, Vec<GenericDef>)>,

    // generic def span to func def span (or struct def span) map
    pub generic_def_span_rev: HashMap<Span, Span>,

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

    // If the programmer calls a generic function, either the programmer has to
    // annotate type, or the compiler has to infer it.
    // It's also a `type_var -> type_annotation` map. It works like `Session.types`, but for generic instances.
    pub generic_instances: HashMap<(Span, Span), Type>,

    // We need this when we create error messages.
    // This is really expensive to initialize, so think twice before you init this.
    pub span_string_map: Option<HashMap<Span, InternedString>>,

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
            ).collect(),
            struct_shapes: hir_session.structs.iter().map(
                |r#struct| (
                    r#struct.name_span,
                    (
                        r#struct.fields.iter().map(
                            |field| StructField {
                                name: field.name,
                                name_span: field.name_span,
                                r#type: None,
                                default_value: field.default_value,
                            }
                        ).collect(),
                        r#struct.generics.clone(),
                    ),
                )
            ).collect(),
            generic_def_span_rev: HashMap::new(),
            lets: vec![],
            funcs: vec![],
            asserts: vec![],
            types: HashMap::new(),
            generic_instances: HashMap::new(),
            span_string_map: Some(HashMap::new()),
            errors: hir_session.errors.clone(),
            warnings: hir_session.warnings.clone(),
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
