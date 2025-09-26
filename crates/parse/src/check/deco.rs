use super::check_call_args;
use crate::{Attribute, Decorator};
use sodigy_error::Error;

impl Attribute {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // TODO: how about doc comments?

        for decorator in self.decorators.iter() {
            if let Err(e) = decorator.check() {
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

impl Decorator {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

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
