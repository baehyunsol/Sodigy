use crate::{Func, FuncArgDef};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Func {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];
        let mut span_by_name: HashMap<InternedString, Span> = HashMap::new();

        if let Err(e) = self.attribute.check() {
            errors.extend(e);
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

            match span_by_name.entry(arg.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: arg.name,
                        },
                        span: arg.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(arg.name_span);
                },
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

        if let Err(e) = self.attribute.check() {
            errors.extend(e);
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
