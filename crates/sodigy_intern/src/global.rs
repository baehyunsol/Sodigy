use crate::{string::{DOTDOTDOT, STRING_B, STRING_F}, InternedString, InternedNumeric};
use sodigy_keyword::keywords;
use sodigy_number::SodigyNumber;
use std::collections::HashMap;
use std::sync::Mutex;

static mut LOCK: Mutex<()> = Mutex::new(());
static mut IS_INIT: bool = false;
pub static mut GLOBAL: *mut GlobalInternSession = std::ptr::null_mut();

unsafe fn init_global() {
    if IS_INIT {
        return;
    }

    let lock = LOCK.lock();
    let mut g = Box::new(GlobalInternSession::new());
    GLOBAL = g.as_mut() as *mut GlobalInternSession;
    std::mem::forget(g);
    IS_INIT = true;
    drop(lock);
}

pub unsafe fn global_intern_session() -> &'static mut GlobalInternSession {
    if !IS_INIT {
        init_global();
    }

    GLOBAL.as_mut().unwrap()
}

pub struct GlobalInternSession {
    pub(crate) strings: HashMap<Vec<u8>, InternedString>,
    pub(crate) strings_rev: HashMap<InternedString, Vec<u8>>,

    pub(crate) numerics: HashMap<SodigyNumber, InternedNumeric>,
    pub(crate) numerics_rev: HashMap<InternedNumeric, SodigyNumber>,
}

impl GlobalInternSession {
    pub fn new() -> Self {
        let mut strings = HashMap::new();
        let mut strings_rev = HashMap::new();
        let numerics = HashMap::new();
        let numerics_rev = HashMap::new();

        for (index, keyword) in keywords().into_iter().enumerate() {
            strings.insert(keyword.to_utf8(), (index as u32).into());
            strings_rev.insert((index as u32).into(), keyword.to_utf8());
        }

        strings.insert(b"b".to_vec(), STRING_B.into());
        strings_rev.insert(STRING_B.into(), b"b".to_vec());
        strings.insert(b"f".to_vec(), STRING_F.into());
        strings_rev.insert(STRING_F.into(), b"f".to_vec());
        strings.insert(b"...".to_vec(), DOTDOTDOT.into());
        strings_rev.insert(DOTDOTDOT.into(), b"...".to_vec());

        GlobalInternSession {
            strings, strings_rev,
            numerics, numerics_rev,
        }
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        unsafe {
            let lock = LOCK.lock();

            match self.strings.get(&string) {
                Some(ii) => *ii,
                None => {
                    let ii = self.get_new_string_index();

                    self.strings.insert(string.clone(), ii);
                    self.strings_rev.insert(ii, string);

                    drop(lock);

                    ii
                },
            }
        }
    }

    fn get_new_string_index(&self) -> InternedString {
        InternedString(self.strings.len() as u32 | 0xff00_0000)
    }

    pub fn intern_numeric(&mut self, numeric: SodigyNumber) -> InternedNumeric {
        unsafe {
            let lock = LOCK.lock();

            match self.numerics.get(&numeric) {
                Some(ii) => *ii,
                None => {
                    let ii = self.get_new_numeric_index();

                    self.numerics.insert(numeric.clone(), ii);
                    self.numerics_rev.insert(ii, numeric);

                    drop(lock);

                    ii
                },
            }
        }
    }

    fn get_new_numeric_index(&self) -> InternedNumeric {
        InternedNumeric(self.numerics.len() as u32 | 0xff00_0000)
    }
}
