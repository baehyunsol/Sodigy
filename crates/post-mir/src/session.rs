use sodigy_error::{Error, Warning};
use sodigy_mir::{GlobalContext, Session as MirSession, Type};
use sodigy_parse::Field;
use sodigy_span::Span;

pub struct Session<'hir, 'mir> {
    pub intermediate_dir: String,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub global_context: GlobalContext<'hir, 'mir>,
}

impl Session<'_, '_> {
    pub fn from_mir_session<'hir, 'mir>(mir_session: &MirSession<'hir, 'mir>) -> Session<'hir, 'mir> {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            errors: vec![],
            warnings: vec![],
            global_context: mir_session.global_context,
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.global_context.lang_items.unwrap().get(lang_item) {
            Some(span) => *span,
            None => panic!("lang_item {lang_item:?}"),
        }
    }

    pub fn get_type_of_field(&mut self, r#type: &Type, field: &[Field]) -> Result<Type, ()> {
        // We need a tmp-type-solver..
        // Or, we can implement a simpler version of `get_type_of_field`, which does not require a full type-solver
        todo!()

        // self.type_solver.get_type_of_field(r#type, field, self.global_context.types.unwrap(), &HashMap::new())
    }
}
