use crate::{Endec, EndecError};
use sodigy_intern::{
    InternedNumeric,
    InternedString,
    InternSession,
    unintern_numeric,
    unintern_string,
};
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

    // when saving encoded data to a file,
    // first write `self.encoded_metadata()` to the file, then
    // write the encoded data
    pub fn encode_metadata(&self) -> Vec<u8> {
        let mut result = vec![];
        let mut dummy_session = EndecSession::new();

        // `str_map` and `str_map_rev` are unnecessary for decoding
        self.str_table.encode(&mut result, &mut dummy_session);
        self.num_table.encode(&mut result, &mut dummy_session);

        result
    }

    // when loading encoded data from a file,
    // first construct `Self` from decoding the file, then
    // start loading the actual data
    pub fn decode_metadata(buf: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        let mut dummy_session = EndecSession::new();
        let mut intern_session = InternSession::new();

        let str_table = HashMap::<EncodedInternal, Vec<u8>>::decode(buf, index, &mut dummy_session)?;
        let mut str_map = HashMap::with_capacity(str_table.len());
        let mut str_map_rev = HashMap::with_capacity(str_table.len());

        for (enc, s) in str_table.iter() {
            let interned_string = intern_session.intern_string(s.to_vec());

            str_map.insert(interned_string, *enc);
            str_map_rev.insert(*enc, interned_string);
        }

        let num_table = HashMap::<EncodedInternal, SodigyNumber>::decode(buf, index, &mut dummy_session)?;
        let mut num_map = HashMap::with_capacity(num_table.len());
        let mut num_map_rev = HashMap::with_capacity(num_table.len());

        for (enc, s) in num_table.iter() {
            let interned_numeric = intern_session.intern_numeric(s.clone());

            num_map.insert(interned_numeric, *enc);
            num_map_rev.insert(*enc, interned_numeric);
        }

        Ok(EndecSession {
            str_table,
            str_map,
            str_map_rev,
            num_table,
            num_map,
            num_map_rev,
        })
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

    pub fn decode_intern_str(&self, e: EncodedInternal) -> Result<InternedString, EndecError> {
        self.str_map_rev.get(&e).map(|i| *i).ok_or_else(|| EndecError::invalid_interned_string())
    }

    pub fn decode_intern_num(&self, e: EncodedInternal) -> Result<InternedNumeric, EndecError> {
        self.num_map_rev.get(&e).map(|i| *i).ok_or_else(|| EndecError::invalid_interned_numeric())
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct EncodedInternal(u32);

impl Endec for EncodedInternal {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(EncodedInternal(u32::decode(buf, index, session)?))
    }
}

impl From<usize> for EncodedInternal {
    fn from(n: usize) -> EncodedInternal {
        EncodedInternal(n as u32)
    }
}
