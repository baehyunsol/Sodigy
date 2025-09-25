use crate::Let;
use sodigy_error::Error;

impl Let {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        for decorator in self.decorators.iter() {
            if let Err(e) = decorator.check() {
                errors.extend(e);
            }
        }

        if let Some(r#type) = &self.r#type {
            if let Err(e) = r#type.check() {
                errors.extend(e);
            }
        }

        if let Err(e) = self.value.check() {
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
