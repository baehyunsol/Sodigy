use super::InternedString;
use crate::token::Keyword;
use std::collections::HashMap;

fn keywords() -> Vec<Vec<u8>> {
    vec![
        b"if".to_vec(),
        b"else".to_vec(),
        b"def".to_vec(),
        b"use".to_vec(),
        b"as".to_vec(),
    ]
}

pub const KEYWORDS: [Keyword; 5] = [
    Keyword::If,
    Keyword::Else,
    Keyword::Def,
    Keyword::Use,
    Keyword::As,
];

pub struct LocalParseSession {
    strings: HashMap<InternedString, Vec<u8>>,
    strings_rev: HashMap<Vec<u8>, InternedString>,
    pub(crate) curr_file: u32,
    pub(crate) is_dummy: bool,

    // no files, but just a direct input
    #[cfg(test)]
    curr_file_data: Vec<u8>,
}

impl LocalParseSession {
    pub fn new() -> Self {
        let keywords = keywords();
        let mut strings = HashMap::with_capacity(keywords.len());
        let mut strings_rev = HashMap::with_capacity(keywords.len());

        for (index, keyword) in keywords.iter().enumerate() {
            strings.insert(index.into(), keyword.to_vec());
            strings_rev.insert(keyword.to_vec(), index.into());
        }

        LocalParseSession {
            strings,
            strings_rev,
            curr_file: u32::MAX, // null
            is_dummy: false,

            #[cfg(test)]
            curr_file_data: vec![],
        }
    }

    pub fn dummy() -> Self {
        LocalParseSession {
            strings: HashMap::new(),
            strings_rev: HashMap::new(),
            curr_file: 0,
            is_dummy: true,
            #[cfg(test)]
            curr_file_data: vec![],
        }
    }

    #[cfg(test)]
    pub fn set_input(&mut self, input: Vec<u8>) {
        self.curr_file_data = input;
    }

    pub fn try_unwrap_keyword(&self, index: InternedString) -> Option<Keyword> {
        KEYWORDS.get::<usize>(index.into()).map(|k| *k)
    }

    // Expensive (if it has to write to a GlobalCache)
    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        match self.strings_rev.get(&string) {
            Some(n) => *n,
            _ => {
                // TODO: first, try to get from the Global cache
                // if fail, make a new entry in the Glocal cache, and get that
                let index = self.strings.len().into();
                self.strings.insert(index, string.clone());
                self.strings_rev.insert(string.clone(), index);
                index
            }
        }
    }

    pub fn unintern_string(&self, string: InternedString) -> Option<Vec<u8>> {
        match self.strings.get(&string) {
            Some(buf) => Some(buf.to_vec()),
            None => {
                #[cfg(test)]
                return None;

                // TODO: search global cache
                #[cfg(not(test))]
                return None;
            }
        }
    }

    // Expensive!
    pub fn get_file_raw_content(&self, index: u32) -> &[u8] {
        #[cfg(test)]
        return &self.curr_file_data;

        #[cfg(not(test))]
        todo!();
    }
}
