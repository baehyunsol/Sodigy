use sodigy_error::{Error, Warning};
use sodigy_hir::StructShape;
use sodigy_inter_mir::TypeSolver;
use sodigy_mir::{Session as MirSession, Type};
use sodigy_parse::Field;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,
    pub struct_shapes: HashMap<Span, StructShape>,
    pub types: HashMap<Span, Type>,
    pub lang_items: HashMap<String, Span>,

    // post-mir doesn't solve types, but it has to call `get_type_of_field`, which requires a type solver.
    pub type_solver: TypeSolver,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_mir_session(mir_session: &MirSession) -> Session {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            struct_shapes: mir_session.struct_shapes.clone(),
            types: mir_session.types.clone(),
            lang_items: mir_session.lang_items.clone(),
            type_solver: TypeSolver::new(
                mir_session.func_shapes.clone(),
                mir_session.struct_shapes.clone(),
                mir_session.lang_items.clone(),
                mir_session.intermediate_dir.to_string(),
            ),
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn get_type_of_field(&mut self, r#type: &Type, field: &[Field]) -> Result<Type, ()> {
        self.type_solver.get_type_of_field(r#type, field, &self.types, &HashMap::new())
    }
}
