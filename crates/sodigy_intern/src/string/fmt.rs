use super::InternedString;
use crate::unintern_string;
use std::fmt;

impl InternedString {
    pub fn render_error(&self) -> String {
        let mut v = unintern_string(*self);

        if v.len() > 64 {
            // TODO: what if v has multi-byte chars?
            v = vec![
                v[..8].to_vec(),
                b"...".to_vec(),
                v[(v.len() - 8)..].to_vec()
            ].concat();
        }

        String::from_utf8_lossy(&v).to_string()
    }

    pub fn escaped_no_quotes(&self) -> String {
        let s = format!("{self:?}").as_bytes().to_vec();

        // first and the last chars are quotes
        String::from_utf8_lossy(&s[1..(s.len() - 1)]).to_string()
    }
}

impl fmt::Display for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let v = unintern_string(*self);
        let s = String::from_utf8_lossy(&v).to_string();

        write!(fmt, "{s}")
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let v = unintern_string(*self);

        write!(fmt, "{:?}", String::from_utf8_lossy(&v).to_string())
    }
}
