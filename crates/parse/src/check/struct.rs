use crate::{Session, Struct};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Struct {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        if let Err(e) = self.attribute.check(session) {
            errors.extend(e);
        }

        if self.fields.is_empty() {
            errors.push(Error {
                kind: ErrorKind::StructWithoutField,
                spans: self.name_span.simple_error(),
                ..Error::default()
            });
        }

        for field in self.fields.iter() {
            if let Err(e) = field.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(field.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(field.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![field.name_span]);
                },
            }
        }

        for (name, spans) in spans_by_name.iter() {
            if spans.len() > 1 {
                errors.push(Error {
                    kind: ErrorKind::NameCollision {
                        name: *name,
                    },
                    spans: spans.iter().map(
                        |span| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect(),
                    ..Error::default()
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
