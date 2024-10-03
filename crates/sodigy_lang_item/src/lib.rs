#![deny(unused_imports)]

use lazy_static::lazy_static;
use sodigy_intern::{InternedString, InternSession};
use sodigy_uid::Uid;
use std::collections::HashMap;

// it has to start with a character that cannot be used by user code
pub const LANG_ITEM_PREFIX: &'static [u8] = b"@@lang_item_";
pub const LANG_ITEMS: [LangItem; 4] = [
    LangItem::Type,
    LangItem::EnumBody,
    LangItem::StructBody,
    LangItem::Dummy,
];

lazy_static! {
    pub static ref LANG_ITEM_MAP: HashMap<Uid, InternedString> = {
        let mut result = HashMap::with_capacity(LANG_ITEMS.len());
        let mut intern_session = InternSession::new();

        for item in LANG_ITEMS.iter() {
            let id = item.into_interned_string(&mut intern_session);
            let uid = id.try_get_lang_item_uid(&mut intern_session).unwrap();

            result.insert(uid, id);
        }

        result
    };
}

pub enum LangItem {
    Type,

    // an enum variant is converted to a function definition by the compiler
    // this is the body of the function
    EnumBody,

    // a struct constructor is converted to a function by the compiler
    // this is the function
    StructBody,

    // it's used when an expr is expected but there's nothing to use
    // for example, `let Option<T>: Type = @@lang_item_dummy`
    // this value is not supposed to be evaluated at runtime
    Dummy,
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
            LangItem::EnumBody => "enum_variant_body",
            LangItem::StructBody => "struct_body",
            LangItem::Dummy => "dummy",
        }
    }
}

// extension on InternedString for LangItem-related methods
pub trait LangItemTrait {
    fn try_get_lang_item_uid(&self, intern_session: &mut InternSession) -> Option<Uid>;
}

impl LangItemTrait for InternedString {
    // TODO: is it okay to use a deterministic hash for uid?
    //       the compiler assumes that *ALL* the uids are unique
    fn try_get_lang_item_uid(&self, intern_session: &mut InternSession) -> Option<Uid> {
        let uninterned = intern_session.unintern_string(*self);

        if uninterned.starts_with(LANG_ITEM_PREFIX) {
            Some(Uid::new_lang_item_from_hash(hash_bytes(&uninterned)))
        }

        else {
            None
        }
    }
}

// I just felt like writing my own hash function
pub fn hash_bytes(bytes: &[u8]) -> u128 {
    let mut result: u128 = 0;

    for (i, c) in bytes.iter().enumerate() {
        let inter = ((result & 0xfff_ffff) << 24) | ((i & 0xfff) << 12) as u128 | *c as u128;
        result += ((inter * inter) << 1) + inter;
    }

    result & 0xffff_ffff_ffff_ffff_ffff_ffff_ffff
}
