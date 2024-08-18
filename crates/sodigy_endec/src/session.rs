use crate::{Endec, EndecError};
use sodigy_files::{
    DUMMY_FILE_HASH,
    exists,
    global_file_session,
};
use sodigy_intern::{
    InternedNumeric,
    InternedString,
    InternSession,
    unintern_numeric,
    unintern_string,
};
use sodigy_number::SodigyNumber;
use std::collections::HashMap;

type FileHash = u64;
type Path = String;

pub struct EndecSession {
    str_map: HashMap<InternedString, EncodedInternal>,
    str_map_rev: HashMap<EncodedInternal, InternedString>,
    str_table: HashMap<EncodedInternal, Vec<u8>>,

    num_map: HashMap<InternedNumeric, EncodedInternal>,
    num_map_rev: HashMap<EncodedInternal, InternedNumeric>,
    num_table: HashMap<EncodedInternal, SodigyNumber>,

    file_hashes: HashMap<FileHash, Path>,
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
            file_hashes: HashMap::new(),
        }
    }

    // when saving encoded data to a file,
    // first write `self.encode_session()` to the file, then
    // write the encoded data
    pub fn encode_session(&self) -> Vec<u8> {
        let mut result = vec![];
        let mut dummy_session = EndecSession::new();

        // the other tables are unnecessary for decoding
        self.str_table.encode(&mut result, &mut dummy_session);
        self.num_table.encode(&mut result, &mut dummy_session);
        self.file_hashes.encode(&mut result, &mut dummy_session);

        result
    }

    // when loading encoded data from a file,
    // first construct `Self` from decoding the file, then
    // start loading the actual data
    pub fn decode_session(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        let mut dummy_session = EndecSession::new();
        let mut intern_session = InternSession::new();

        let str_table = HashMap::<EncodedInternal, Vec<u8>>::decode(buffer, index, &mut dummy_session)?;
        let mut str_map = HashMap::with_capacity(str_table.len());
        let mut str_map_rev = HashMap::with_capacity(str_table.len());

        for (enc, s) in str_table.iter() {
            let interned_string = intern_session.intern_string(s.to_vec());

            str_map.insert(interned_string, *enc);
            str_map_rev.insert(*enc, interned_string);
        }

        let num_table = HashMap::<EncodedInternal, SodigyNumber>::decode(buffer, index, &mut dummy_session)?;
        let mut num_map = HashMap::with_capacity(num_table.len());
        let mut num_map_rev = HashMap::with_capacity(num_table.len());

        for (enc, s) in num_table.iter() {
            let interned_numeric = intern_session.intern_numeric(s.clone());

            num_map.insert(interned_numeric, *enc);
            num_map_rev.insert(*enc, interned_numeric);
        }

        let file_hashes = HashMap::<FileHash, Path>::decode(buffer, index, &mut dummy_session)?;
        let file_session = unsafe { global_file_session() };

        for (hash, path) in file_hashes.iter() {
            // we do this existence check because the file might have been moved (that affects the relative path)
            if !exists(path) {
                return Err(EndecError::corrupted_file_hash(*hash, path.to_string()));
            }

            file_session.try_register_hash_and_file(*hash, path)?;
        }

        Ok(EndecSession {
            str_table,
            str_map,
            str_map_rev,
            num_table,
            num_map,
            num_map_rev,
            file_hashes,
        })
    }

    pub fn register_file_hash(&mut self, file: FileHash) {
        if !self.file_hashes.contains_key(&file) && file != DUMMY_FILE_HASH {
            let file_session = unsafe { global_file_session() };
            self.file_hashes.insert(file, file_session.get_file_name_from_hash(file).unwrap());
        }
    }

    pub fn encode_intern_str(&mut self, s: InternedString) -> EncodedInternal {
        match self.str_map.get(&s) {
            Some(s) => *s,
            None => {
                let n: EncodedInternal = self.str_map.len().into();

                self.str_map.insert(s, n);
                self.str_map_rev.insert(n, s);
                self.str_table.insert(n, unintern_string(s));

                n
            },
        }
    }

    pub fn encode_intern_num(&mut self, n: InternedNumeric) -> EncodedInternal {
        match self.num_map.get(&n) {
            Some(n) => *n,
            None => {
                let nn: EncodedInternal = self.num_map.len().into();

                self.num_map.insert(n, nn);
                self.num_map_rev.insert(nn, n);
                self.num_table.insert(nn, unintern_numeric(n));

                nn
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
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(EncodedInternal(u32::decode(buffer, index, session)?))
    }
}

impl From<usize> for EncodedInternal {
    fn from(n: usize) -> EncodedInternal {
        EncodedInternal(n as u32)
    }
}
