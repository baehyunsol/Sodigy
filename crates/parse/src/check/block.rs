use crate::{Block, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

impl Block {
    pub fn check(&self, is_top_level: bool, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        // for error messages
        let mut let_spans = HashSet::new();

        for r#let in self.lets.iter() {
            if let Err(e) = r#let.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(r#let.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(r#let.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![r#let.name_span]);
                },
            }

            let_spans.insert(r#let.name_span);
        }

        for func in self.funcs.iter() {
            if let Err(e) = func.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(func.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(func.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![func.name_span]);
                },
            }
        }

        for r#struct in self.structs.iter() {
            if let Err(e) = r#struct.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(r#struct.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(r#struct.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![r#struct.name_span]);
                },
            }
        }

        for r#enum in self.enums.iter() {
            if let Err(e) = r#enum.check() {
                errors.extend(e);
            }

            match spans_by_name.entry(r#enum.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(r#enum.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![r#enum.name_span]);
                },
            }
        }

        for module in self.modules.iter() {
            if !is_top_level {
                errors.push(Error {
                    kind: ErrorKind::CannotDeclareInlineModule,
                    spans: module.keyword_span.simple_error(),
                    ..Error::default()
                });
            }

            if let Err(e) = module.check() {
                errors.extend(e);
            }

            match spans_by_name.entry(module.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(module.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![module.name_span]);
                },
            }
        }

        for r#use in self.uses.iter() {
            if let Err(e) = r#use.check() {
                errors.extend(e);
            }

            match spans_by_name.entry(r#use.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(r#use.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![r#use.name_span]);
                },
            }
        }

        for (name, spans) in spans_by_name.iter() {
            if spans.len() > 1 {
                let note = if spans.iter().all(|span| let_spans.contains(span)) {
                    Some(String::from("You cannot shadow names in Sodigy."))
                } else {
                    None
                };

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
                    note,
                });
            }
        }

        if let Some(value) = self.value.as_ref() {
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
