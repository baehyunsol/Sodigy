use crate::err::HirError;

pub struct HirSession {
    errors: Vec<HirError>,
}

impl HirSession {
    pub fn new() -> Self {
        HirSession {
            errors: vec![],
        }
    }

    pub fn push_error(&mut self, error: HirError) {
        self.errors.push(error);
    }

    pub fn get_errors(&self) -> &Vec<HirError> {
        &self.errors
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
