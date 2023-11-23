use crate::{Endec, EndecErr};
use sodigy_intern::{InternedNumeric, InternedString, unintern_numeric, unintern_string};
use sodigy_number::SodigyNumber;
use std::collections::HashMap;

pub struct EndecSession {
    str_map: HashMap<InternedString, EncodedInternal>,
    str_map_rev: HashMap<EncodedInternal, InternedString>,
    str_table: HashMap<EncodedInternal, Vec<u8>>,

    num_map: HashMap<InternedNumeric, EncodedInternal>,
    num_map_rev: HashMap<EncodedInternal, InternedNumeric>,
    num_table: HashMap<EncodedInternal, SodigyNumber>,
}

impl EndecSession {
    pub fn new() -> Self {
        EndecSession {
            str_map: HashMap::new(),
            str_map_rev: HashMap::new(),
            str_table: HashMap::new(),
            num_map: HashMap::new(),
            num_map_rev: HashMap::new(),
            num_table: HashMap::new(),
        }
    }

    pub fn encode_intern_str(&mut self, s: InternedString) -> EncodedInternal {
        match self.str_map.get(&s) {
            Some(s) => *s,
            None => {
                let n: EncodedInternal = self.str_map.len().into();

                self.str_map.insert(s, n);
                self.str_table.insert(n, unintern_string(s));

                n
            },
        }
    }

    pub fn encode_intern_num(&mut self, s: InternedNumeric) -> EncodedInternal {
        match self.num_map.get(&s) {
            Some(s) => *s,
            None => {
                let n: EncodedInternal = self.num_map.len().into();

                self.num_map.insert(s, n);
                self.num_table.insert(n, unintern_numeric(s));

                n
            },
        }
    }

    pub fn decode_intern_str(&self, e: EncodedInternal) -> Result<InternedString, EndecErr> {
        self.str_map_rev.get(&e).map(|i| *i).ok_or_else(|| EndecErr::InvalidInternedString)
    }

    pub fn decode_intern_num(&self, e: EncodedInternal) -> Result<InternedNumeric, EndecErr> {
        self.num_map_rev.get(&e).map(|i| *i).ok_or_else(|| EndecErr::InvalidInternedNumeric)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct EncodedInternal(u32);

impl Endec for EncodedInternal {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(EncodedInternal(u32::decode(buf, ind, session)?))
    }
}

impl From<usize> for EncodedInternal {
    fn from(n: usize) -> EncodedInternal {
        EncodedInternal(n as u32)
    }
}
