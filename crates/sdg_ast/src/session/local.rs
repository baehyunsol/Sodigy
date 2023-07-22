use super::{GLOBAL_SESSION, GLOBAL_SESSION_LOCK, InternedString, KEYWORDS, KEYWORD_START, try_init_global_session};
use crate::token::Keyword;
use crate::warning::SodigyWarning;
use sdg_fs::read_bytes;
use std::collections::HashMap;

#[derive(Default)]
pub struct LocalParseSession {
    strings: HashMap<InternedString, Vec<u8>>,
    strings_rev: HashMap<Vec<u8>, InternedString>,
    pub(crate) curr_file: u64,
    pub(crate) is_dummy: bool,

    warnings: Vec<SodigyWarning>,

    // only for test purpose!
    // don't use `#[cfg(test)]`: I want it to work on other crates
    curr_file_data: Vec<u8>,
}

impl LocalParseSession {
    pub fn new() -> Self {
        try_init_global_session();

        LocalParseSession {
            curr_file: u64::MAX, // null
            is_dummy: false,
            ..Self::default()
        }
    }

    pub fn dummy() -> Self {
        LocalParseSession {
            curr_file: 0,
            is_dummy: true,
            ..Self::default()
        }
    }

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
                let result = unsafe {
                    let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error CB9665F9D46");
                    let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 77C4E2EDBE9");

                    let r = g.intern_string(string.clone());
                    drop(lock);

                    r
                };

                self.strings.insert(result, string.clone());
                self.strings_rev.insert(string.clone(), result);

                result
            }
        }
    }

    // It succeeds if `string` is already interned by this local session
    pub fn try_intern_string(&self, string: Vec<u8>) -> Option<InternedString> {
        self.strings_rev.get(&string).map(|s| *s)
    }

    pub fn unintern_string(&self, string: InternedString) -> Vec<u8> {
        match self.strings.get(&string) {
            Some(buf) => buf.to_vec(),
            None => {
                unsafe {
                    let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error CB9665F9D46");
                    let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 77C4E2EDBE9");

                    let r = g.unintern_string(string);
                    drop(lock);

                    r
                }
            }
        }
    }

    pub fn add_warning(&mut self, warning: SodigyWarning) {
        self.warnings.push(warning);
    }

    pub fn add_warnings(&mut self, warnings: Vec<SodigyWarning>) {
        for warning in warnings.into_iter() {
            self.warnings.push(warning);
        }
    }

    pub fn get_file_path(&self, index: u64) -> String {
        #[cfg(test)]
        return "tests/test.sdg".to_string();

        // TODO: cache this in the local session!
        #[cfg(not(test))]
        return unsafe {
            let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error 9C9003FC163");
            let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 721788AA0BA");

            let p = g.get_file_path(index);
            drop(lock);

            p
        };
    }

    // Expensive!
    pub fn get_file_raw_content(&self, index: u64) -> Vec<u8> {
        if !self.curr_file_data.is_empty() {
            self.curr_file_data.clone()
        }

        else {
            let path = self.get_file_path(index);

            // What do we do here? There's no way the compiler can recover from this
            read_bytes(&path).expect("Internal Compiler Error D4A59FCCCE0")
        }
    }
}
