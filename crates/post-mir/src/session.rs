use sodigy_error::{Error, Warning};
use sodigy_inter_mir::TypeSolver;
use sodigy_mir::{GlobalContext, Session as MirSession, Type};
use sodigy_parse::Field;
use std::collections::HashMap;

pub struct Session<'hir, 'mir> {
    pub intermediate_dir: String,

    // post-mir doesn't solve types, but it has to call `get_type_of_field`, which requires a type solver.
    pub type_solver: TypeSolver<'hir, 'mir>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub global_context: GlobalContext<'hir, 'mir>,
}

impl Session<'_, '_> {
    pub fn from_mir_session<'hir, 'mir>(mir_session: &MirSession<'hir, 'mir>) -> Session<'hir, 'mir> {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            type_solver: TypeSolver::new(mir_session.global_context, mir_session.intermediate_dir.to_string()),
            errors: vec![],
            warnings: vec![],
            global_context: mir_session.global_context,
        }
    }

    pub fn get_type_of_field(&mut self, r#type: &Type, field: &[Field]) -> Result<Type, ()> {
        self.type_solver.get_type_of_field(r#type, field, self.global_context.types.unwrap(), &HashMap::new())
    }
}
