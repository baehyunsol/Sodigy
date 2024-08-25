#![deny(unused_imports)]

use sodigy_intern::{InternedString, InternSession};

// it has to start with a character that cannot be used by user code
pub const LANG_ITEM_PREFIX: &'static [u8] = b"@@__lang_item_";

pub enum LangItem {
    Type,

    // it's not a 'real' lang item
    // it's used when a compiler feature is not implemented, but I don't want the compiler to panic
    Todo,
}

impl LangItem {
    pub fn into_interned_string(&self, intern_session: &mut InternSession) -> InternedString {
        intern_session.intern_string(format!(
            "{}{}",
            unsafe { String::from_utf8_unchecked(LANG_ITEM_PREFIX.to_vec()) },
            self.into_sodigy_name(),
        ).bytes().collect())
    }

    pub fn into_sodigy_name(&self) -> &'static str {
        match self {
            LangItem::Type => "type",
            LangItem::Todo => "todo",
        }
    }
}

// extension on InternedString for LangItem-related methods
pub trait LangItemTrait {
    fn is_lang_item(&self, intern_session: &mut InternSession) -> bool;
}

impl LangItemTrait for InternedString {
    fn is_lang_item(&self, intern_session: &mut InternSession) -> bool {
        if let Some(s) = intern_session.unintern_string(*self) {
            s.starts_with(LANG_ITEM_PREFIX)
        }

        else {
            false
        }
    }
}
