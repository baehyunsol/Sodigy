use crate::MatchDump;
use sodigy_error::{Error, Warning};
use sodigy_mir::{GlobalContext, Session as MirSession, Type};
use sodigy_session::SodigySession;
use sodigy_span::{Span, SpanId};
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Session<'hir, 'mir> {
    pub intermediate_dir: String,
    pub match_dumps: Option<Vec<MatchDump>>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub global_context: GlobalContext<'hir, 'mir>,
}

impl<'hir, 'mir> Session<'hir, 'mir> {
    pub fn from_mir_session(
        mir_session: &MirSession<'hir, 'mir>,
        match_dumps: bool,
    ) -> Self {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            match_dumps: if match_dumps { Some(vec![]) } else { None },
            errors: vec![],
            warnings: vec![],
            global_context: mir_session.global_context.clone(),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.global_context.lang_items.unwrap().get(lang_item) {
            Some(span) => span.clone(),
            None => panic!("lang_item {lang_item:?}"),
        }
    }

    pub fn add_type_info(&mut self, def_span: &Span, r#type: Type) {
        self.global_context.types
            .as_mut()
            .expect("global context not initialized")
            .write()
            .expect("global context poisoned")
            .insert(def_span.clone(), r#type);
    }

    pub fn get_variant_index(&self, parent: &Span, variant: &Span) -> Option<u32> {
        self.global_context.enum_shapes.unwrap().get(parent).map(|enum_shape| enum_shape.get_variant_index(variant))?
    }
}

impl SodigySession for Session<'_, '_> {
    fn intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }

    fn lang_items(&self) -> Option<&HashMap<String, Span>> {
        self.global_context.lang_items
    }

    fn span_string_map(&self) -> Option<&HashMap<SpanId, InternedString>> {
        self.global_context.span_string_map
    }

    fn variant_to_enum_span(&self) -> Option<&HashMap<Span, Span>> {
        self.global_context.variant_to_enum_span
    }
}
