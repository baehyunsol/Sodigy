use sodigy_span::{Span, SpanId};
use sodigy_string::InternedString;
use std::collections::HashMap;

pub trait SodigySession {
    fn intermediate_dir(&self) -> &str;

    fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items() {
            Some(lang_items) => match lang_items.get(lang_item) {
                Some(s) => s.clone(),
                None => panic!("TODO: lang_item `{lang_item}`"),
            },
            None => panic!("session.lang_items is not initialized yet"),
        }
    }

    fn lang_items(&self) -> Option<&HashMap<String, Span>> {
        None
    }

    fn span_string_map(&self) -> Option<&HashMap<SpanId, InternedString>> {
        None
    }
}
