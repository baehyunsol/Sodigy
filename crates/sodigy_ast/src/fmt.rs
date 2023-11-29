use super::TypeDef;
use std::fmt;

impl fmt::Display for TypeDef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.0)
    }
}
