use crate::{Enum, EnumVariant};
use sodigy_error::{Error, ErrorKind, NameCollisionKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Enum {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_variant_name: HashMap<InternedString, Vec<Span>> = HashMap::new();
        let mut spans_by_generic_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        if let Err(e) = self.attribute.check(intermediate_dir) {
            errors.extend(e);
        }

        for generic in self.generics.iter() {
            match spans_by_generic_name.entry(generic.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(generic.name_span.clone());
                },
                Entry::Vacant(e) => {
                    e.insert(vec![generic.name_span.clone()]);
                },
            }
        }

        if let Some(variants) = &self.variants {
            for variant in variants.iter() {
                if let Err(e) = variant.check(intermediate_dir) {
                    errors.extend(e);
                }

                match spans_by_variant_name.entry(variant.name) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(variant.name_span.clone());
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![variant.name_span.clone()]);
                    },
                }
            }
        }

        for (name, spans, is_variant) in spans_by_variant_name.iter().map(|(name, spans)| (name, spans, true)).chain(spans_by_generic_name.iter().map(|(name, spans)| (name, spans, false))) {
            if spans.len() > 1 {
                errors.push(Error {
                    kind: ErrorKind::NameCollision {
                        name: *name,
                        kind: if is_variant { NameCollisionKind::Enum } else { NameCollisionKind::EnumGeneric },
                    },
                    spans: spans.iter().map(
                        |span| RenderableSpan {
                            span: span.clone(),
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
