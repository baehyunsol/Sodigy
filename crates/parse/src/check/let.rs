use crate::{Let, Session};
use sodigy_error::Error;

impl Let {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.attribute.check(session) {
            errors.extend(e);
        }

        if let Some(type_annot) = &self.type_annot {
            if let Err(e) = type_annot.check() {
                errors.extend(e);
            }
        }

        if let Err(e) = self.value.check(session) {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(())
        }

        else {
            Err(errors)
        }
    }
}
