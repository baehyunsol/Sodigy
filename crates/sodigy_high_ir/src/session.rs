use crate::err::HirError;
use crate::warn::HirWarning;
use sodigy_intern::InternedString;
use std::collections::HashSet;

pub struct HirSession {
    errors: Vec<HirError>,
    warnings: Vec<HirWarning>,
}

impl HirSession {
    pub fn new() -> Self {
        HirSession {
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn get_prelude_names(&self) -> HashSet<InternedString> {
        HashSet::new()
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

    pub fn err_if_has_err(&self) -> Result<(), ()> {
        if self.errors.is_empty() {
            Ok(())
        }

        else {
            Err(())
        }
    }
}
