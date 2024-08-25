#![deny(unused_imports)]

use sodigy_intern::{InternedString, InternSession};

pub enum LangItem {
    Type,

    // it's not a 'real' lang item
    // it's used when a compiler feature is not implemented, but I don't want the compiler to panic
    Todo,
}

impl LangItem {
    pub fn into_interned_string(&self, intern_session: &mut InternSession) -> InternedString {
        intern_session.intern_string(format!("@@__lang_item_{}", self.into_sodigy_name()).bytes().collect())
    }

    pub fn into_sodigy_name(&self) -> &'static str {
        match self {
            LangItem::Type => "type",
            LangItem::Todo => "todo",
        }
    }
}
