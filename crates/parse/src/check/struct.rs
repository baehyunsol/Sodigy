use crate::Struct;
use sodigy_error::{Error, ErrorKind};

impl Struct {
    // TODO: name collision checks
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.attribute.check() {
            errors.extend(e);
        }

        if self.fields.is_empty() {
            errors.push(Error {
                kind: ErrorKind::StructWithoutField,
                span: self.name_span,
                ..Error::default()
            });
        }

        for field in self.fields.iter() {
            if let Err(e) = field.check() {
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
