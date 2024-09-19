use super::Type;
use std::fmt;

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            Type::HasToBeInferred => String::from("_"),
        };

        write!(fmt, "{s}")
    }
}
