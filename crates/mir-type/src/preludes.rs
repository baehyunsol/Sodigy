use sodigy_string::{InternedString, intern_string};

// NOTE: I want `const INT: InternedString = intern_string(b"Int");`,
//       but Rust doesn't allow me to do that. It doesn't even allow me to
//       call `try_intern_short_string` in const context.

pub(crate) const INT: usize = 0;
pub(crate) const NUMBER: usize = 1;
pub(crate) const BOOL: usize = 2;
pub(crate) const STRING: usize = 3;
pub(crate) const BYTES: usize = 4;
pub(crate) const LIST: usize = 5;
pub(crate) const CHAR: usize = 6;
pub(crate) const BYTE: usize = 7;

pub fn get_preludes() -> Vec<InternedString> {
    vec![
        intern_string(b"Int", "").unwrap(),
        intern_string(b"Number", "").unwrap(),
        intern_string(b"Bool", "").unwrap(),
        intern_string(b"String", "").unwrap(),
        intern_string(b"Bytes", "").unwrap(),
        intern_string(b"List", "").unwrap(),
        intern_string(b"Char", "").unwrap(),
        intern_string(b"Byte", "").unwrap(),
    ]
}
