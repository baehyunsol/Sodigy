use crate::{Match, MatchArm};
use sodigy_error::{Error, ErrorKind};

impl Match {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.arms.is_empty() {
            errors.push(Error {
                kind: ErrorKind::EmptyMatchStatement,
                spans: self.keyword_span.simple_error(),
                note: None,
            });
        }

        for arm in self.arms.iter() {
            if let Err(e) = arm.check(intermediate_dir) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        }

        else {
            Err(errors)
        }
    }
}

impl MatchArm {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.pattern.check(/* is_inner_pattern: */ false, intermediate_dir) {
            errors.extend(e);
        }

        if let Some(guard) = &self.guard {
            if let Err(e) = guard.check(intermediate_dir) {
                errors.extend(e);
            }
        }

        if let Err(e) = self.value.check(intermediate_dir) {
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
