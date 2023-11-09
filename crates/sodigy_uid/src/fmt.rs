use super::*;
use std::fmt;

impl fmt::Display for Uid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "uid#{}",
            self.0,
        )
    }
}

impl fmt::Debug for Uid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let ty = match self.get_type() as u128 {
            x if x == (DEF >> 124) => "DEF",
            x if x == (ENUM >> 124) => "ENUM",
            x if x == (STRUCT >> 124) => "STRUCT",
            x if x == (MODULE >> 124) => "MODULE",
            x if x == (LAMBDA >> 124) => "LAMBDA",
            x if x == (SCOPE_BLOCK >> 124) => "SCOPE_BLOCK",
            x if x == (MATCH_ARM >> 124) => "MATCH_ARM",
            _ => "UNKNOWN_TYPE",
        };

        let prelude = if self.is_prelude() {
            "prelude"
        } else {
            "not_prelude"
        };

        let data = self.0 & !(0xff >> 120);

        write!(
            fmt,
            "Uid({ty}, {prelude}, {data:x})"
        )
    }
}
