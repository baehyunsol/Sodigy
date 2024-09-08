use crate::{
    global::global_intern_session,
    InternedNumeric,
    InternedString,
    numeric::try_intern_small_integer,
    string::try_intern_short_string,
};
use sodigy_keyword::keywords;
use sodigy_number::SodigyNumber;
use std::collections::HashMap;

#[derive(Clone)]
pub struct LocalInternSession {
    pub(crate) local_string_table: HashMap<Vec<u8>, InternedString>,
    pub(crate) local_string_table_rev: HashMap<InternedString, Vec<u8>>,

    pub(crate) local_numeric_table: HashMap<SodigyNumber, InternedNumeric>,
    pub(crate) local_numeric_table_rev: HashMap<InternedNumeric, SodigyNumber>,
}

impl LocalInternSession {
    pub fn new() -> Self {
        let keywords = keywords();

        let mut local_string_table = HashMap::with_capacity(keywords.len());
        let mut local_string_table_rev = HashMap::with_capacity(keywords.len());

        for (index, keyword) in keywords.into_iter().enumerate() {
            local_string_table.insert(keyword.to_utf8(), (index as u32).into());
            local_string_table_rev.insert((index as u32).into(), keyword.to_utf8());
        }

        LocalInternSession {
            local_string_table,
            local_string_table_rev,
            local_numeric_table: HashMap::new(),
            local_numeric_table_rev: HashMap::new(),
        }
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        match self.local_string_table.get(&string) {
            Some(i) => *i,
            None => unsafe {
                let ii = if let Some(s) = try_intern_short_string(&string) {
                    return s;
                } else {
                    let g = global_intern_session();

                    g.intern_string(string.clone())
                };

                self.local_string_table.insert(string.clone(), ii);
                self.local_string_table_rev.insert(ii, string.clone());

                ii
            },
        }
    }

    pub fn intern_numeric(&mut self, numeric: SodigyNumber) -> InternedNumeric {
        match self.local_numeric_table.get(&numeric) {
            Some(i) => *i,
            None => unsafe {
                let ii = match u32::try_from(&numeric) {
                    Ok(n) if let Some(n) = try_intern_small_integer(n) => {
                        return n;
                    },
                    _ => {
                        let g = global_intern_session();

                        g.intern_numeric(numeric.clone())
                    },
                };

                self.local_numeric_table.insert(numeric.clone(), ii);
                self.local_numeric_table_rev.insert(ii, numeric.clone());

                ii
            },
        }
    }

    /// it only searches the local session
    pub fn unintern_string_fast(&self, string: InternedString) -> Option<&[u8]> {
        self.local_string_table_rev.get(&string).map(|s| s as &[u8])
    }

    pub fn unintern_string(&mut self, string: InternedString) -> &[u8] {
        match self.unintern_string_fast(string) {
            // trust me, it's safe
            Some(s) => unsafe { std::mem::transmute::<&[u8], &'static [u8]>(s) },
            None => unsafe {
                if let Some((length, bytes)) = string.try_unwrap_short_string() {
                    let s = bytes[0..(length as usize)].to_vec();

                    self.local_string_table.insert(s.clone(), string);
                    self.local_string_table_rev.insert(string, s.clone());

                    self.unintern_string(string)
                }

                else {
                    let g = global_intern_session();

                    // if it fails, it's an ICE
                    g.strings_rev.get(&string).map(|s| s as &[u8]).unwrap()
                }
            },
        }
    }

    /// it only searches the local session
    pub fn unintern_numeric_fast(&self, numeric: InternedNumeric) -> Option<&SodigyNumber> {
        self.local_numeric_table_rev.get(&numeric)
    }

    pub fn unintern_numeric(&mut self, numeric: InternedNumeric) -> &SodigyNumber {
        match self.unintern_numeric_fast(numeric) {
            // trust me, it's safe
            Some(n) => unsafe { std::mem::transmute::<&SodigyNumber, &'static SodigyNumber>(n) },
            None => unsafe {
                if let Some(n) = numeric.try_unwrap_small_integer() {
                    let n = SodigyNumber::SmallInt(n as i64);

                    self.local_numeric_table.insert(n.clone(), numeric);
                    self.local_numeric_table_rev.insert(numeric, n.clone());

                    self.unintern_numeric(numeric)
                }

                else {
                    let g = global_intern_session();

                    // if it fails, it's an ICE
                    g.numerics_rev.get(&numeric).unwrap()
                }
            },
        }
    }
}
