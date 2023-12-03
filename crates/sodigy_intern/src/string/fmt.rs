use super::InternedString;
use crate::unintern_string;
use std::fmt;

impl InternedString {
    pub fn render_error(&self) -> String {
        let mut v = unintern_string(*self);

        if v.len() > 64 {
            v = vec![
                first_few_chars(&v),
                b"...".to_vec(),
                last_few_chars(&v),
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

// v.len() > 64, v is a valid utf-8 str
fn first_few_chars(v: &[u8]) -> Vec<u8> {
    let mut curr = 7;

    loop {
        if v[curr] < 128 {
            return v[0..curr].to_vec();
        }

        else if v[curr] >= 192 {
            return v[0..(curr - 1)].to_vec();
        }

        curr += 1;
    }
}

// v.len() > 64, v is a valid utf-8 str
fn last_few_chars(v: &[u8]) -> Vec<u8> {
    let mut curr = v.len() - 8;

    loop {
        if v[curr] < 128 || v[curr] >= 192 {
            return v[curr..].to_vec();
        }

        curr -= 1;
    }
}
