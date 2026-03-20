use sodigy_error::{Error, Warning};
use sodigy_mir::{GlobalContext, Session as MirSession, Type};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session<'hir, 'mir> {
    pub intermediate_dir: String,
    pub match_dumps: Option<HashMap<Span, (Vec<(Span, String)>, String)>>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub global_context: GlobalContext<'hir, 'mir>,
}

impl Session<'_, '_> {
    pub fn from_mir_session<'hir, 'mir>(
        mir_session: &MirSession<'hir, 'mir>,
        match_dumps: bool,
    ) -> Session<'hir, 'mir> {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            match_dumps: if match_dumps { Some(HashMap::new()) } else { None },
            errors: vec![],
            warnings: vec![],
            global_context: mir_session.global_context.clone(),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.global_context.lang_items.unwrap().get(lang_item) {
            Some(span) => *span,
            None => panic!("lang_item {lang_item:?}"),
        }
    }

    pub fn add_type_info(&mut self, def_span: Span, r#type: Type) {
        self.global_context.types
            .as_mut()
            .expect("global context not initialized")
            .write()
            .expect("global context poisoned")
            .insert(def_span, r#type);
    }
}
