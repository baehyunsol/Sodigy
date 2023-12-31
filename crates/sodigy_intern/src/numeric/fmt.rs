use super::InternedNumeric;
use crate::unintern_numeric;
use std::fmt;

impl fmt::Debug for InternedNumeric {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            unintern_numeric(*self),
        )
    }
}

impl fmt::Display for InternedNumeric {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            unintern_numeric(*self),
        )
    }
}
