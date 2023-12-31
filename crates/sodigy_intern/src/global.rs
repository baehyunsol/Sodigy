use crate::{
    InternedNumeric,
    InternedString,
    numeric::try_intern_small_integer,
    prelude::DATA_MASK,
    string::try_intern_short_string,
};
use sodigy_keyword::keywords;
use sodigy_number::SodigyNumber;
use std::collections::HashMap;
use std::sync::Mutex;

static mut LOCK: Mutex<()> = Mutex::new(());
static mut IS_INIT: bool = false;
static mut GLOBAL: *mut GlobalInternSession = std::ptr::null_mut();

unsafe fn init_global() {
    if IS_INIT {
        return;
    }

    let lock = LOCK.lock().unwrap();

    // very rare situation: two threads enters this function at the same time,
    // check `IS_INIT`, which are both false, then only one of them acquires lock
    // in order to handle this case, it has to check `IS_INIT` after acquiring the lock
    // we still need the first lock, because that reduces overhead in most cases
    if IS_INIT {
        return;
    }

    let mut g = Box::new(GlobalInternSession::new());
    GLOBAL = g.as_mut() as *mut GlobalInternSession;
    std::mem::forget(g);
    IS_INIT = true;
    drop(lock);
}

pub(crate) unsafe fn global_intern_session() -> &'static mut GlobalInternSession {
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
    fn new() -> Self {
        let mut strings = HashMap::new();
        let mut strings_rev = HashMap::new();
        let numerics = HashMap::new();
        let numerics_rev = HashMap::new();

        for (index, keyword) in keywords().into_iter().enumerate() {
            strings.insert(keyword.to_utf8(), (index as u32).into());
            strings_rev.insert((index as u32).into(), keyword.to_utf8());
        }

        GlobalInternSession {
            strings, strings_rev,
            numerics, numerics_rev,
        }
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        unsafe {
            let lock = LOCK.lock().unwrap();

            match self.strings.get(&string) {
                Some(ii) => *ii,
                None => {
                    if let Some(s) = try_intern_short_string(&string) {
                        drop(lock);
                        s
                    }

                    else {
                        let ii = self.get_new_string_index();

                        self.strings.insert(string.clone(), ii);
                        self.strings_rev.insert(ii, string);

                        drop(lock);
                        ii
                    }
                },
            }
        }
    }

    fn get_new_string_index(&self) -> InternedString {
        let data = self.strings.len() as u32 & DATA_MASK;

        InternedString(data)
    }

    pub fn intern_numeric(&mut self, numeric: SodigyNumber) -> InternedNumeric {
        unsafe {
            let lock = LOCK.lock().unwrap();

            match self.numerics.get(&numeric) {
                Some(ii) => *ii,
                None => match u32::try_from(&numeric) {
                    Ok(n) if let Some(nn) = try_intern_small_integer(n) => {
                        drop(lock);

                        nn
                    },
                    _ => {
                        let ii = self.get_new_numeric_index();

                        self.numerics.insert(numeric.clone(), ii);
                        self.numerics_rev.insert(ii, numeric);

                        drop(lock);

                        ii
                    },
                },
            }
        }
    }

    fn get_new_numeric_index(&self) -> InternedNumeric {
        let data = self.numerics.len() as u32 & DATA_MASK;

        InternedNumeric(data)
    }
}
