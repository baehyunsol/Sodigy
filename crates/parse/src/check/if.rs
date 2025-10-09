use crate::If;
use sodigy_error::Error;

impl If {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.cond.check() {
            errors.extend(e);
        }

        if let Some(pattern) = &self.pattern {
            if let Err(e) = pattern.check(
                /* allow type annotation: */ false,
                /* check name collision: */ true,
            ) {
                errors.extend(e);
            }
        }

        if let Err(e) = self.true_value.check() {
            errors.extend(e);
        }

        if let Err(e) = self.false_value.check() {
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
