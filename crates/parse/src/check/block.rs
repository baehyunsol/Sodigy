use crate::Block;
use sodigy_error::{Error, ErrorKind, NameCollisionKind};
use sodigy_name_analysis::NameKind;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Block {
    pub fn check(&self, is_top_level: bool, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // name collision check
        let mut spans_by_name: HashMap<InternedString, Vec<(Span, NameKind)>> = HashMap::new();

        for assert in self.asserts.iter() {
            if let Err(e) = assert.value.check(intermediate_dir) {
                errors.extend(e);
            }

            if let Err(e) = assert.attribute.check(intermediate_dir) {
                errors.extend(e);
            }
        }

        for r#let in self.lets.iter() {
            if let Err(e) = r#let.check(intermediate_dir) {
                errors.extend(e);
            }

            if !r#let.name.eq(b"_") {
                match spans_by_name.entry(r#let.name) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push((r#let.name_span, NameKind::Let { is_top_level }));
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![(r#let.name_span, NameKind::Let { is_top_level })]);
                    },
                }
            }
        }

        for func in self.funcs.iter() {
            if let Err(e) = func.check(intermediate_dir) {
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
            if let Err(e) = r#struct.check(intermediate_dir) {
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
            if let Err(e) = r#enum.check(intermediate_dir) {
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
                    note: None,
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
                    let name_rendered = name.unintern_or_default(intermediate_dir);

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
                        kind: NameCollisionKind::Block { is_top_level },
                    },
                    spans,
                    note,
                });
            }
        }

        if let Some(value) = self.value.as_ref() {
            if let Err(e) = value.check(intermediate_dir) {
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
