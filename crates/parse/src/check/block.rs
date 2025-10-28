use crate::{Block, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::NameKind;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, unintern_string};
use std::collections::hash_map::{Entry, HashMap};

impl Block {
    pub fn check(&self, is_top_level: bool, session: &Session) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_name: HashMap<InternedString, Vec<(Span, NameKind)>> = HashMap::new();

        for r#let in self.lets.iter() {
            if let Err(e) = r#let.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(r#let.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push((r#let.name_span, NameKind::Let { is_top_level }));
                },
                Entry::Vacant(e) => {
                    e.insert(vec![(r#let.name_span, NameKind::Let { is_top_level })]);
                },
            }
        }

        for func in self.funcs.iter() {
            if let Err(e) = func.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(func.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push((func.name_span, NameKind::Func));
                },
                Entry::Vacant(e) => {
                    e.insert(vec![(func.name_span, NameKind::Func)]);
                },
            }
        }

        for r#struct in self.structs.iter() {
            if let Err(e) = r#struct.check(session) {
                errors.extend(e);
            }

            match spans_by_name.entry(r#struct.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push((r#struct.name_span, NameKind::Struct));
                },
                Entry::Vacant(e) => {
                    e.insert(vec![(r#struct.name_span, NameKind::Struct)]);
                },
            }
        }

        for r#enum in self.enums.iter() {
            if let Err(e) = r#enum.check() {
                errors.extend(e);
            }

            match spans_by_name.entry(r#enum.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push((r#enum.name_span, NameKind::Enum));
                },
                Entry::Vacant(e) => {
                    e.insert(vec![(r#enum.name_span, NameKind::Enum)]);
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
                    e.get_mut().push((module.name_span, NameKind::Module));
                },
                Entry::Vacant(e) => {
                    e.insert(vec![(module.name_span, NameKind::Module)]);
                },
            }
        }

        for r#use in self.uses.iter() {
            if let Err(e) = r#use.check() {
                errors.extend(e);
            }

            match spans_by_name.entry(r#use.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push((r#use.name_span, NameKind::Use));
                },
                Entry::Vacant(e) => {
                    e.insert(vec![(r#use.name_span, NameKind::Use)]);
                },
            }
        }

        for (name, spans) in spans_by_name.iter() {
            if spans.len() > 1 {
                let note = if spans.iter().all(|(_, kind)| matches!(kind, NameKind::Let { .. })) {
                    Some(String::from("You cannot shadow names in Sodigy."))
                } else {
                    None
                };
                let spans = if spans.len() == 2 &&
                    spans.iter().any(|(_, kind)| *kind == NameKind::Use) &&
                    spans.iter().any(|(_, kind)| *kind == NameKind::Module)
                {
                    let (use_span, module_span) = if spans[0].1 == NameKind::Use {
                        (spans[0].0, spans[1].0)
                    } else {
                        (spans[1].0, spans[0].0)
                    };
                    let name_rendered = unintern_string(*name, &session.intermediate_dir).unwrap().unwrap();
                    let name_rendered = String::from_utf8_lossy(&name_rendered).to_string();

                    vec![
                        RenderableSpan {
                            span: module_span,
                            auxiliary: false,
                            note: Some(format!("`mod {name_rendered};` implicitly imports the name `{name_rendered}` into the namespace.")),
                        },
                        RenderableSpan {
                            span: use_span,
                            auxiliary: false,
                            note: None,
                        },
                    ]
                } else {
                    spans.iter().map(
                        |(span, _)| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect()
                };

                errors.push(Error {
                    kind: ErrorKind::NameCollision {
                        name: *name,
                    },
                    spans,
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
