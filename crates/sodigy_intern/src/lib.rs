#![deny(unused_imports)]
#![feature(if_let_guard)]

use sodigy_number::SodigyNumber;

mod global;
mod numeric;
mod prelude;
mod session;
mod string;

#[cfg(test)]
mod tests;

pub use numeric::{InternedNumeric, try_intern_small_integer};
pub use string::{InternedString, try_intern_short_string};

pub use session::LocalInternSession as InternSession;

/// If you have a local intern_session, you should prefer using that.
pub fn intern_string(s: Vec<u8>) -> InternedString {
    let g = unsafe { global::global_intern_session() };

    g.intern_string(s)
}

/// If you have a local intern_session, you should prefer using that.
pub fn intern_numeric(n: SodigyNumber) -> InternedNumeric {
    let g = unsafe { global::global_intern_session() };

    g.intern_numeric(n)
}

/// If you have a local intern_session, you should prefer using that.
pub fn intern_numeric_u32(n: u32) -> InternedNumeric {
    if let Some(n) = try_intern_small_integer(n) {
        n
    }

    else {
        intern_numeric(SodigyNumber::SmallInt(n as u64))
    }
}

/// If you have a local intern_session, you should prefer using that.
pub fn unintern_string(s: InternedString) -> Vec<u8> {
    if let Some((length, bytes)) = s.try_unwrap_short_string() {
        bytes[0..(length as usize)].to_vec()
    }

    else {
        let g = unsafe { global::global_intern_session() };

        // if it fails, that's ICE
        g.strings_rev.get(&s).unwrap().to_vec()
    }
}

/// If you have a local intern_session, you should prefer using that.
pub fn unintern_numeric(n: InternedNumeric) -> SodigyNumber {
    if let Some(n) = n.try_unwrap_small_integer() {
        SodigyNumber::SmallInt(n as u64)
    }

    else {
        let g = unsafe { global::global_intern_session() };

        // if it fails, that's ICE
        g.numerics_rev.get(&n).unwrap().clone()
    }
}
