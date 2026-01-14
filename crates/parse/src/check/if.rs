use crate::If;
use sodigy_error::Error;

impl If {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.cond.check(intermediate_dir) {
            errors.extend(e);
        }

        if let Some(pattern) = &self.pattern {
            if let Err(e) = pattern.check(/* is_inner_pattern: */ false, intermediate_dir) {
                errors.extend(e);
            }
        }

        if let Err(e) = self.true_value.check(intermediate_dir) {
            errors.extend(e);
        }

        if let Err(e) = self.false_value.check(intermediate_dir) {
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
