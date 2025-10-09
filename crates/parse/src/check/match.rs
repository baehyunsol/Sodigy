use crate::{Match, MatchBranch};
use sodigy_error::Error;

impl Match {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

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
            /* check name collision: */ true,
        )
    }
}
