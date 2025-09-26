use crate::Block;
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Block {
    // TODO: sort the errors by span
    pub fn check(&self, top_level: bool) -> Result<(), Vec<Error>> {
        let mut errors = vec![];
        let mut span_by_name: HashMap<InternedString, Span> = HashMap::new();

        for r#let in self.lets.iter() {
            if let Err(e) = r#let.check() {
                errors.extend(e);
            }

            match span_by_name.entry(r#let.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: r#let.name,
                        },
                        span: r#let.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(r#let.name_span);
                },
            }
        }

        for func in self.funcs.iter() {
            if let Err(e) = func.check() {
                errors.extend(e);
            }

            match span_by_name.entry(func.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: func.name,
                        },
                        span: func.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(func.name_span);
                },
            }
        }

        for r#struct in self.structs.iter() {
            if let Err(e) = r#struct.check() {
                errors.extend(e);
            }

            match span_by_name.entry(r#struct.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: r#struct.name,
                        },
                        span: r#struct.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(r#struct.name_span);
                },
            }
        }

        for r#enum in self.enums.iter() {
            if let Err(e) = r#enum.check() {
                errors.extend(e);
            }

            match span_by_name.entry(r#enum.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: r#enum.name,
                        },
                        span: r#enum.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(r#enum.name_span);
                },
            }
        }

        for module in self.modules.iter() {
            if !top_level {
                errors.push(Error {
                    kind: ErrorKind::CannotDeclareInlineModule,
                    span: module.keyword_span,
                    ..Error::default()
                });
            }

            if let Err(e) = module.check() {
                errors.extend(e);
            }

            match span_by_name.entry(module.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: module.name,
                        },
                        span: module.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(module.name_span);
                },
            }
        }

        for r#use in self.uses.iter() {
            if let Err(e) = r#use.check() {
                errors.extend(e);
            }

            match span_by_name.entry(r#use.name) {
                Entry::Occupied(e) => {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: r#use.name,
                        },
                        span: r#use.name_span,
                        extra_span: Some(*e.get()),
                        ..Error::default()
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(r#use.name_span);
                },
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
