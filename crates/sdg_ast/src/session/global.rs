use super::{InternedString, KEYWORD_START};
use sdg_prelude::{get_prelude_buffs, get_prelude_index};
use crate::token::Keyword;
use sdg_hash::SdgHash;
use std::collections::HashMap;

pub const DUMMY_FILE_INDEX: u64 = u64::MAX;

pub struct GlobalParseSession {
    strings: HashMap<InternedString, Vec<u8>>,
    strings_rev: HashMap<Vec<u8>, InternedString>,
    files: HashMap<u64, String>,
    files_rev: HashMap<String, u64>,
}

pub static mut GLOBAL_SESSION: *mut GlobalParseSession = std::ptr::null_mut();
pub static mut GLOBAL_SESSION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl GlobalParseSession {
    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        match self.strings_rev.get(&string) {
            Some(i) => *i,
            None => {
                let index = self.strings.len() + KEYWORD_START as usize;
                let index: InternedString = index.into();

                self.strings.insert(index, string.clone());
                self.strings_rev.insert(string.clone(), index);

                index
            }
        }
    }

    pub fn unintern_string(&self, string: InternedString) -> Vec<u8> {
        self.strings.get(&string).expect("Internal Compiler Error E634CD266D0").to_vec()
    }

    // It registers the path to `files` and returns the index.
    // If the path is already registered, it just returns the index from `files`.
    pub fn register_file(&mut self, path: &str) -> u64 {
        match self.files_rev.get(path) {
            Some(i) => *i,
            _ => {
                let mut index = path.sdg_hash().to_u64();

                // avoid hash collision
                while self.files.contains_key(&index) || index == DUMMY_FILE_INDEX {
                    index = index.sdg_hash().to_u64();
                }

                self.files.insert(index, path.to_string());
                self.files_rev.insert(path.to_string(), index);

                index
            }
        }
    }

    pub fn get_file_path(&self, index: u64) -> String {
        match self.files.get(&index) {
            Some(f) => f.to_string(),

            _ => {
                assert_eq!(index, DUMMY_FILE_INDEX, "Internal Compiler Error 4F5423CF234");

                String::from("./tests/tests.sdg")
            },
        }
    }
}

fn keywords() -> Vec<Vec<u8>> {
    KEYWORDS.iter().map(|k| k.render_err().as_bytes().to_vec()).collect()
}

pub const KEYWORDS: [Keyword; 7] = [
    Keyword::If,
    Keyword::Else,
    Keyword::Def,
    Keyword::Use,
    Keyword::As,
    Keyword::Let,
    Keyword::Module,
];

pub fn try_init_global_session() {
    unsafe {
        let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error 0DDC04FBD91");

        if GLOBAL_SESSION.is_null() {
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

            let result = Box::new(GlobalParseSession {
                strings,
                strings_rev,
                files: HashMap::new(),
                files_rev: HashMap::new(),
            });
            GLOBAL_SESSION = Box::leak(result) as *mut GlobalParseSession;

            drop(lock);
        }

    }
}
