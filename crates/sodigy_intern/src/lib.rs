mod global;
mod string;
mod numeric;
mod prelude;
mod session;

#[cfg(test)]
mod tests;

pub use string::InternedString;
pub use numeric::InternedNumeric;

pub use session::Session as InternSession;
pub use prelude::*;

/// This function is very expensive. Please use this function only for test purpose.
/// If you have a local intern session, use `Session.unintern_string` instead of this one.
pub fn unintern_string(s: InternedString) -> Vec<u8> {
    let g = unsafe { global::global_intern_session() };

    g.strings_rev.get(&s).unwrap().to_vec()
}
