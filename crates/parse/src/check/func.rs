use crate::{Func, FuncParam, Session};
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
        let mut span_of_param_with_default_value = None;

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

        for param in self.params.iter() {
            if let Some(span) = span_of_param_with_default_value && param.default_value.is_none() {
                errors.push(Error {
                    kind: ErrorKind::NonDefaultValueAfterDefaultValue,
                    spans: vec![
                        RenderableSpan {
                            span: param.name_span,
                            auxiliary: false,
                            note: Some(String::from("This parameter must have a default value.")),
                        },
                        RenderableSpan {
                            span,
                            auxiliary: true,
                            note: Some(String::from("This parameter has a default value.")),
                        },
                    ],
                    note: None,
                });
            }

            if let Err(e) = param.check(session) {
                errors.extend(e);
            }

            if param.default_value.is_some() {
                span_of_param_with_default_value = Some(param.name_span);
            }

            match spans_by_name.entry(param.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(param.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![param.name_span]);
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
                    note: None,
                });
            }
        }

        if let Some(type_annot) = &self.type_annot {
            if let Err(e) = type_annot.check() {
                errors.extend(e);
            }
        }

        if let Some(value) = &self.value {
            if let Err(e) = value.check(session) {
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

impl FuncParam {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.attribute.check(session) {
            errors.extend(e);
        }

        if let Some(type_annot) = &self.type_annot {
            if let Err(e) = type_annot.check() {
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
