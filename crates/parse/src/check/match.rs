use crate::{Match, MatchArm, Session};
use sodigy_error::{Error, ErrorKind};

impl Match {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.arms.is_empty() {
            errors.push(Error {
                kind: ErrorKind::EmptyMatchStatement,
                spans: self.keyword_span.simple_error(),
                note: None,
            });
        }

        for arm in self.arms.iter() {
            if let Err(e) = arm.check(session) {
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
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        self.pattern.check(
            /* allow type annotation: */ false,
            /* is_inner_pattern: */ false,
            session,
        )
    }
}
