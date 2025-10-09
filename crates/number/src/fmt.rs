use super::{InternedNumber, InternedNumberValue};
use std::fmt;

impl fmt::Debug for InternedNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self.value {
            InternedNumberValue::SmallInteger(n) => format!(
                "SmallInteger({n}{})",
                if self.is_integer { "" } else { ".0" },
            ),
            InternedNumberValue::SmallRatio { numer, denom } => format!("SmallRatio({numer} / {denom})"),
        };

        write!(fmt, "{s}")
    }
}
