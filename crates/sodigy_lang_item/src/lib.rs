#![deny(unused_imports)]

use sodigy_intern::InternedString;

pub enum LangItem {
    Type,
}

impl LangItem {
    pub fn into_interned_string(&self) -> InternedString {
        todo!()
    }
}
