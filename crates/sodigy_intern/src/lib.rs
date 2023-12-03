#![deny(unused_imports)]

use sodigy_number::SodigyNumber;

mod global;
mod numeric;
mod prelude;
mod session;
mod string;

pub(crate) use global::{DATA_MASK, IS_INTEGER, IS_SMALL_INTEGER, PRELUDE_STRINGS};
pub use numeric::InternedNumeric;
pub use string::InternedString;

pub use session::Session as InternSession;

/// This function is very expensive. Please use this function only for test purpose.
pub fn intern_string(s: Vec<u8>) -> InternedString {
    let g = unsafe { global::global_intern_session() };

    g.intern_string(s)
}

/// This function is very expensive. Please use this function only for test purpose.
pub fn intern_numeric(n: SodigyNumber) -> InternedNumeric {
    let g = unsafe { global::global_intern_session() };

    g.intern_numeric(n)
}

/// This function is very expensive. Please use this function only for test purpose.
/// If you have a local intern session, use `Session.unintern_string_fast` instead of this one.
pub fn unintern_string(s: InternedString) -> Vec<u8> {
    let g = unsafe { global::global_intern_session() };

    g.strings_rev.get(&s).unwrap().to_vec()
}

pub fn unintern_numeric(n: InternedNumeric) -> SodigyNumber {
    let g = unsafe { global::global_intern_session() };

    g.numerics_rev.get(&n).unwrap().clone()
}
