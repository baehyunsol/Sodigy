use crate::{If, Session};
use sodigy_error::Error;

impl If {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.cond.check(session) {
            errors.extend(e);
        }

        if let Some(pattern) = &self.pattern {
            if let Err(e) = pattern.check(/* is_inner_pattern: */ false, session) {
                errors.extend(e);
            }
        }

        if let Err(e) = self.true_value.check(session) {
            errors.extend(e);
        }

        if let Err(e) = self.false_value.check(session) {
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
