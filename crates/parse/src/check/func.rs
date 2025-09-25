use crate::{Func, FuncArgDef};
use sodigy_error::{Error, ErrorKind};

impl Func {
    // TODO: name collision checks
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        for decorator in self.decorators.iter() {
            if let Err(e) = decorator.check() {
                errors.extend(e);
            }
        }

        let mut must_have_default_value = false;

        for arg in self.args.iter() {
            if must_have_default_value && arg.default_value.is_none() {
                errors.push(Error {
                    kind: ErrorKind::NonDefaultValueAfterDefaultValue,
                    span: arg.name_span,
                    ..Error::default()
                });
            }

            if let Err(e) = arg.check() {
                errors.extend(e);
            }

            if arg.default_value.is_some() {
                must_have_default_value = true;
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

impl FuncArgDef {
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

        if let Some(default_value) = &self.default_value {
            if let Err(e) = default_value.check() {
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
