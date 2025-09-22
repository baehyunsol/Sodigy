use super::InternedNumber;
use std::fmt;

impl fmt::Debug for InternedNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            InternedNumber::SmallInteger(n) => format!("SmallInteger({n})"),
            InternedNumber::SmallRatio { numer, denom } => format!("SmallRatio({numer} / {denom})"),
        };

        write!(fmt, "{s}")
    }
}
