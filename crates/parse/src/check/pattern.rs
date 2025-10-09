use crate::{FullPattern, Pattern};
use sodigy_error::{Error, ErrorKind};
use std::collections::hash_map::{Entry, HashMap};

impl FullPattern {
    pub fn check(
        &self,
        allow_type_annotation: bool,

        // If patterns are nested, we don't have to check name collisions
        // in the inner patterns.
        check_name_collision: bool,
    ) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if !allow_type_annotation {
            if let Some(r#type) = &self.r#type {
                errors.push(Error {
                    kind: ErrorKind::CannotAnnotateType,
                    span: r#type.error_span(),
                    ..Error::default()
                });
            }
        }

        if check_name_collision {
            let mut name_map = HashMap::new();

            for (name, name_span) in self.bound_names().iter() {
                match name_map.entry(*name) {
                    Entry::Occupied(e) => {
                        let prev_span = *e.get();
                        errors.push(Error {
                            kind: ErrorKind::NameCollision {
                                name: *name,
                            },
                            span: *name_span,
                            extra_span: Some(prev_span),
                            ..Error::default()
                        });
                    },
                    Entry::Vacant(e) => {
                        e.insert(*name_span);
                    },
                }
            }
        }

        if let Err(e) = self.pattern.check() {
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

impl Pattern {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        match self {
            _ => todo!(),
        }
    }
}
