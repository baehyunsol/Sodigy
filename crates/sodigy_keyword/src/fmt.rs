use super::Keyword;
use std::fmt;

impl fmt::Display for Keyword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            format!("{self:?}").to_lowercase(),
        )
    }
}
