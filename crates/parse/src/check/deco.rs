use super::check_call_args;
use crate::Decorator;
use sodigy_error::{Error, ErrorKind};

impl Decorator {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // Like Python, a positional argument cannot follow a keyword argument
        let mut has_to_be_kwarg = false;

        if let Some(args) = &self.args {
            if let Err(e) = check_call_args(args) {
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
