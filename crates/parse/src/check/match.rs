use crate::{Match, MatchBranch, Session};
use sodigy_error::{Error, ErrorKind};

impl Match {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.branches.is_empty() {
            errors.push(Error {
                kind: ErrorKind::EmptyMatchStatement,
                spans: self.keyword_span.simple_error(),
                ..Error::default()
            });
        }

        for branch in self.branches.iter() {
            if let Err(e) = branch.check(session) {
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

impl MatchBranch {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        self.pattern.check(
            /* allow type annotation: */ false,
            /* is_inner_pattern: */ false,
            session,
        )
    }
}
