use crate::Block;
use sodigy_error::Error;

impl Block {
    // TODO: sort the errors by span
    // TODO: name collision checks
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        for r#let in self.lets.iter() {
            if let Err(e) = r#let.check() {
                errors.extend(e);
            }
        }

        for r#func in self.funcs.iter() {
            if let Err(e) = r#func.check() {
                errors.extend(e);
            }
        }

        for r#struct in self.structs.iter() {
            if let Err(e) = r#struct.check() {
                errors.extend(e);
            }
        }

        for r#enum in self.enums.iter() {
            if let Err(e) = r#enum.check() {
                errors.extend(e);
            }
        }

        if let Some(value) = self.value.as_ref() {
            if let Err(e) = value.check() {
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
