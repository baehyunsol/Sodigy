use super::{InternedString, KEYWORD_START};
use crate::prelude::{get_prelude_buffs, get_prelude_index};
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
    pub(crate) curr_file: u64,
    pub(crate) is_dummy: bool,

    // no files, but just a direct input
    #[cfg(test)]
    curr_file_data: Vec<u8>,
}

impl LocalParseSession {
    pub fn new() -> Self {
        let keywords = keywords();
        let preludes = get_prelude_buffs();

        let mut strings = HashMap::with_capacity(keywords.len() + preludes.len());
        let mut strings_rev = HashMap::with_capacity(keywords.len() + preludes.len());

        for (index, keyword) in keywords.iter().enumerate() {
            strings.insert((index + KEYWORD_START as usize).into(), keyword.to_vec());
            strings_rev.insert(keyword.to_vec(), (index + KEYWORD_START as usize).into());
        }

        for (index, prelude) in preludes.iter().enumerate() {
            strings.insert(get_prelude_index(index).into(), prelude.to_vec());
            strings_rev.insert(prelude.to_vec(), get_prelude_index(index).into());
        }

        LocalParseSession {
            strings,
            strings_rev,
            curr_file: u64::MAX, // null
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
        let index: usize = index.into();

        if index >= KEYWORD_START as usize {
            KEYWORDS.get(index - KEYWORD_START as usize).map(|k| *k)
        }

        else {
            None
        }
    }

    // Expensive (if it has to write to a GlobalCache)
    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        match self.strings_rev.get(&string) {
            Some(n) => *n,
            _ => {
                // TODO: first, try to get from the Global cache
                // if fail, make a new entry in the Glocal cache, and get that
                let index = self.strings.len() + KEYWORD_START as usize;
                let index: InternedString = index.into();

                self.strings.insert(index, string.clone());
                self.strings_rev.insert(string.clone(), index);

                index
            }
        }
    }

    // It succeeds if `string` is already interned
    pub fn try_intern_string(&self, string: Vec<u8>) -> Option<InternedString> {
        self.strings_rev.get(&string).map(|s| *s)
    }

    pub fn unintern_string(&self, string: InternedString) -> Vec<u8> {
        match self.strings.get(&string) {
            Some(buf) => buf.to_vec(),
            None => {
                // TODO: search global cache
                // it must be somewhere!
                todo!()
            }
        }
    }

    pub fn get_file_path(&self, index: u64) -> Vec<u8> {
        #[cfg(test)]
        return b"tests/test.sdg".to_vec();

        #[cfg(not(test))]
        todo!();
    }

    // Expensive!
    pub fn get_file_raw_content(&self, index: u64) -> &[u8] {
        #[cfg(test)]
        return &self.curr_file_data;

        #[cfg(not(test))]
        todo!();
    }
}
