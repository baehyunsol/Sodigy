use super::Type;
use std::fmt;

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.0)
    }
}