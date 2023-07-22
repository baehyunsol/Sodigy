use super::{InternedString, KEYWORD_START};
use crate::prelude::{get_prelude_buffs, get_prelude_index};
use crate::token::Keyword;
use std::collections::HashMap;

pub struct GlobalParseSession {
    strings: HashMap<InternedString, Vec<u8>>,
    strings_rev: HashMap<Vec<u8>, InternedString>,
    files: HashMap<u64, String>,
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

    pub fn get_file_path(&self, index: u64) -> String {
        match self.files.get(&index) {
            Some(f) => f.to_string(),
            _ => self.files.get(&index).map(|s| s.to_string()).unwrap_or(String::from("./TODO/What/Do/I/Do/In/This/Case"))
        }
    }
}

fn keywords() -> Vec<Vec<u8>> {
    vec![
        b"if".to_vec(),
        b"else".to_vec(),
        b"def".to_vec(),
        b"use".to_vec(),
        b"as".to_vec(),
        b"let".to_vec(),
    ]
}

pub const KEYWORDS: [Keyword; 6] = [
    Keyword::If,
    Keyword::Else,
    Keyword::Def,
    Keyword::Use,
    Keyword::As,
    Keyword::Let,
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
                files: HashMap::new(),  // TODO: How do we initialize this?
            });
            GLOBAL_SESSION = Box::leak(result) as *mut GlobalParseSession;

            drop(lock);
        }

    }
}
