use crate::Let;
use sodigy_error::Error;

impl Let {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.attribute.check(intermediate_dir) {
            errors.extend(e);
        }

        if let Some(type_annot) = &self.type_annot {
            if let Err(e) = type_annot.check() {
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
