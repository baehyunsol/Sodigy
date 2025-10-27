use crate::{Func, FuncArgDef, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Func {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        if let Err(e) = self.attribute.check(session) {
            errors.extend(e);
        }

        // for error messages
        let mut span_of_arg_with_default_value = None;

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

        for arg in self.args.iter() {
            if let Some(span) = span_of_arg_with_default_value && arg.default_value.is_none() {
                errors.push(Error {
                    kind: ErrorKind::NonDefaultValueAfterDefaultValue,
                    spans: vec![
                        RenderableSpan {
                            span: arg.name_span,
                            auxiliary: false,
                            note: Some(String::from("This argument must have a default value.")),
                        },
                        RenderableSpan {
                            span,
                            auxiliary: true,
                            note: Some(String::from("This argument has a default value.")),
                        },
                    ],
                    note: None,
                });
            }

            if let Err(e) = arg.check(session) {
                errors.extend(e);
            }

            if arg.default_value.is_some() {
                span_of_arg_with_default_value = Some(arg.name_span);
            }

            match spans_by_name.entry(arg.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(arg.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![arg.name_span]);
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

        if let Some(r#type) = &self.r#type {
            if let Err(e) = r#type.check() {
                errors.extend(e);
            }
        }

        if let Err(e) = self.value.check(session) {
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
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.attribute.check(session) {
            errors.extend(e);
        }

        if let Some(r#type) = &self.r#type {
            if let Err(e) = r#type.check() {
                errors.extend(e);
            }
        }

        if let Some(default_value) = &self.default_value {
            if let Err(e) = default_value.check(session) {
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
