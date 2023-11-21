use crate::err::HirError;
use crate::func::Func;
use crate::warn::HirWarning;
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_number::SodigyNumber;
use sodigy_prelude::PRELUDES;
use std::collections::{HashMap, HashSet};

pub struct HirSession {
    errors: Vec<HirError>,
    warnings: Vec<HirWarning>,
    interner: InternSession,

    // HashMap<name, def>
    pub func_defs: HashMap<InternedString, Func>
}

impl HirSession {
    pub fn new() -> Self {
        HirSession {
            errors: vec![],
            warnings: vec![],
            interner: InternSession::new(),
            func_defs: HashMap::new(),
        }
    }

    pub fn get_prelude_names(&self) -> HashSet<InternedString> {
        PRELUDES.keys().map(|k| *k).collect()
    }

    pub fn push_error(&mut self, error: HirError) {
        self.errors.push(error);
    }

    pub fn get_errors(&self) -> &Vec<HirError> {
        &self.errors
    }

    pub fn push_warning(&mut self, warning: HirWarning) {
        self.warnings.push(warning);
    }

    pub fn get_warnings(&self) -> &Vec<HirWarning> {
        &self.warnings
    }

    pub fn intern_numeric(&mut self, n: SodigyNumber) -> InternedNumeric {
        self.interner.intern_numeric(n)
    }

    pub fn unintern_numeric(&mut self, s: InternedNumeric) -> Option<&SodigyNumber> {
        self.interner.unintern_numeric(s)
    }

    pub fn intern_string(&mut self, s: Vec<u8>) -> InternedString {
        self.interner.intern_string(s)
    }

    pub fn unintern_string(&mut self, s: InternedString) -> Option<&[u8]> {
        self.interner.unintern_string(s)
    }

    pub fn err_if_has_err(&self) -> Result<(), ()> {
        if self.errors.is_empty() {
            Ok(())
        }

        else {
            Err(())
        }
    }
}
