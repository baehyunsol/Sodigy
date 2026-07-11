use crate::Struct;
use sodigy_error::{Error, ErrorKind, NameCollisionKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Struct {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_field_name: HashMap<InternedString, Vec<Span>> = HashMap::new();
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

        // Some([]) = self.fields -> Error::StructWithoutField
        // None = self.fields && built_in -> Ok
        // None = self.fields && !built_in -> Error, but hir will report this
        if let Some(fields) = &self.fields && fields.is_empty() {
            errors.push(Error {
                kind: ErrorKind::StructWithoutField,
                spans: self.name_span.simple_error(),
                note: None,
            });
        }

        if let Some(fields) = &self.fields {
            for field in fields.iter() {
                if let Err(e) = field.check(intermediate_dir) {
                    errors.extend(e);
                }

                match spans_by_field_name.entry(field.name) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(field.name_span.clone());
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![field.name_span.clone()]);
                    },
                }
            }
        }

        for (name, spans, is_field) in spans_by_field_name.iter().map(|(name, spans)| (name, spans, true)).chain(spans_by_generic_name.iter().map(|(name, spans)| (name, spans, false))) {
            if spans.len() > 1 {
                errors.push(Error {
                    kind: ErrorKind::NameCollision {
                        name: *name,
                        kind: if is_field { NameCollisionKind::Struct } else { NameCollisionKind::StructGeneric },
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
