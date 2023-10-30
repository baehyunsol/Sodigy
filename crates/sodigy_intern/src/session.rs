use crate::{global::global_intern_session, InternedString, InternedNumeric};
use sodigy_number::SodigyNumber;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Session {
    pub(crate) local_string_table: HashMap<Vec<u8>, InternedString>,
    pub(crate) local_string_table_rev: HashMap<InternedString, Vec<u8>>,

    pub(crate) local_numeric_table: HashMap<SodigyNumber, InternedNumeric>,
    pub(crate) local_numeric_table_rev: HashMap<InternedNumeric, SodigyNumber>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            local_string_table: HashMap::new(),
            local_string_table_rev: HashMap::new(),
            local_numeric_table: HashMap::new(),
            local_numeric_table_rev: HashMap::new(),
        }
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        match self.local_string_table.get(&string) {
            Some(i) => *i,
            None => unsafe {
                let g = global_intern_session();

                let ii = g.intern_string(string.clone());
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
                let g = global_intern_session();

                let ii = g.intern_numeric(numeric.clone());
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

    pub fn unintern_string(&mut self, string: InternedString) -> Option<&[u8]> {
        match self.unintern_string_fast(string) {
            Some(s) => Some(s),
            None => unsafe {
                let g = global_intern_session();

                g.strings_rev.get(&string).map(|s| s as &[u8])
            },
        }
    }

    /// it only searches the local session
    pub fn unintern_numeric_fast(&self, numeric: InternedNumeric) -> Option<&SodigyNumber> {
        self.local_numeric_table_rev.get(&numeric)
    }
}
