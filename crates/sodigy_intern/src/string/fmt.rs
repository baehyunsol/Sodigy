use super::InternedString;
use crate::unintern_string;
use std::fmt;

impl fmt::Display for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut v = unintern_string(*self);

        if v.len() > 64 {
            v = vec![
                v[..8].to_vec(),
                b"...".to_vec(),
                v[(v.len() - 8)..].to_vec()
            ].concat();
        }

        let s = String::from_utf8_lossy(&v).to_string();

        write!(fmt, "{}", s)
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let v = unintern_string(*self);

        write!(fmt, "{:?}", String::from_utf8_lossy(&v).to_string())
    }
}
