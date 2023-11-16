use sodigy_number::SodigyNumber;

mod global;
mod string;
mod numeric;
mod prelude;
mod session;

#[cfg(test)]
mod tests;

pub(crate) use global::{IS_INTEGER, PRELUDE_NUMERICS, PRELUDE_STRINGS};
pub use numeric::InternedNumeric;
pub use string::InternedString;

pub use session::Session as InternSession;

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
