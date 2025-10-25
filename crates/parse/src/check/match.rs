use crate::{Match, MatchBranch};
use sodigy_error::{Error, ErrorKind};

impl Match {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.branches.is_empty() {
            errors.push(Error {
                kind: ErrorKind::EmptyMatchStatement,
                spans: self.keyword_span.simple_error(),
                ..Error::default()
            });
        }

        for branch in self.branches.iter() {
            if let Err(e) = branch.check() {
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
    pub fn check(&self) -> Result<(), Vec<Error>> {
        self.pattern.check(
            /* allow type annotation: */ false,
            /* is_inner_pattern: */ false,
        )
    }
}
