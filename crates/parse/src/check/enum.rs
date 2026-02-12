use crate::{Enum, EnumVariant};
use sodigy_error::{Error, ErrorKind, NameCollisionKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Enum {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        if let Err(e) = self.attribute.check(intermediate_dir) {
            errors.extend(e);
        }

        for generic in self.generics.iter() {
            match spans_by_name.entry(generic.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(generic.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![generic.name_span]);
                },
            }
        }

        if let Some(variants) = &self.variants {
            for variant in variants.iter() {
                if let Err(e) = variant.check(intermediate_dir) {
                    errors.extend(e);
                }

                match spans_by_name.entry(variant.name) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(variant.name_span);
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![variant.name_span]);
                    },
                }
            }
        }

        for (name, spans) in spans_by_name.iter() {
            if spans.len() > 1 {
                errors.push(Error {
                    kind: ErrorKind::NameCollision {
                        name: *name,
                        kind: NameCollisionKind::Enum,
                    },
                    spans: spans.iter().map(
                        |span| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect(),
                    note: None,
                });
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

impl EnumVariant {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        self.attribute.check(intermediate_dir)
    }
}
