use super::InternedString;
use crate::global::global_intern_session;
use std::fmt;

impl fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            unsafe {
                let g = global_intern_session();

                match g.strings_rev.get(self) {
                    Some(s) => String::from_utf8_lossy(s).to_string(),
                    _ => "(ERROR: Uninterned String)".to_string(),
                }
            },
        )
    }
}

impl fmt::Display for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
