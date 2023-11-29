use super::InternedNumeric;
use crate::global::global_intern_session;
use std::fmt;

impl fmt::Debug for InternedNumeric {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            unsafe {
                let g = global_intern_session();

                match g.numerics_rev.get(self) {
                    Some(n) => format!("{n:?}"),
                    _ => "(ERROR: Uninterned Numeric)".to_string(),
                }
            },
        )
    }
}

impl fmt::Display for InternedNumeric {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            unsafe {
                let g = global_intern_session();

                match g.numerics_rev.get(self) {
                    Some(n) => n.to_string(),
                    _ => "(ERROR: Uninterned Numeric)".to_string(),
                }
            },
        )
    }
}
