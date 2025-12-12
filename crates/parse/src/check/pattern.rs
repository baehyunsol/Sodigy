use crate::{Pattern, PatternKind, Session};
use sodigy_error::{Error, ErrorKind, comma_list_strs};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, unintern_string};
use std::collections::hash_map::{Entry, HashMap};

impl Pattern {
    pub fn check(
        &self,
        allow_type_annotation: bool,

        // If patterns are nested, we don't have to check name collisions
        // in the inner patterns. Also, we only type-check the outer-most pattern.
        is_inner_pattern: bool,
        session: &Session,
    ) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if !allow_type_annotation {
            if let Some(r#type) = &self.r#type {
                errors.push(Error {
                    kind: ErrorKind::CannotAnnotateType,
                    spans: r#type.error_span().simple_error(),
                    note: None,
                });
            }
        }

        if !is_inner_pattern {
            let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

            for (name, name_span) in self.bound_names().iter() {
                match spans_by_name.entry(*name) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(*name_span);
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![*name_span]);
                    },
                }
            }

            for (name, spans) in spans_by_name.iter() {
                if spans.len() > 1 {
                    errors.push(Error {
                        kind: ErrorKind::NameCollision {
                            name: *name
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
        }

        if let Err(e) = self.kind.check(session) {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(())
        }

        else {
            Err(errors)
        }
    }

    pub fn check_range_argument(&self, is_lhs: bool) -> Result<(), Vec<Error>> {
        match &self.kind {
            PatternKind::DollarIdent { .. } |
            PatternKind::Number { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } => Ok(()),

            // TODO: If lhs and rhs are all const, it's valid!
            PatternKind::InfixOp { lhs, rhs, .. } => todo!(),

            _ => Err(vec![Error {
                kind: ErrorKind::InvalidRangePattern,
                spans: self.error_span().simple_error(),
                note: if matches!(&self.kind, PatternKind::Wildcard(_)) {
                    Some(format!(
                        "If you want an open-ended range, just leave {} empty instead of using a wildcard.",
                        if is_lhs { "lhs" } else { "rhs" },
                    ))
                } else {
                    None
                },
            }]),
        }
    }
}

impl PatternKind {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        match self {
            PatternKind::Number { .. } |
            PatternKind::String { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } |
            PatternKind::Ident { .. } |
            PatternKind::Path(_) |
            PatternKind::Wildcard(_) |
            PatternKind::DollarIdent { .. } => Ok(()),
            PatternKind::Regex { .. } => todo!(),
            PatternKind::Struct { fields, .. } => {
                // There maybe name collisions in the fields, but AST doesn't care about that.
                let mut errors = vec![];

                for field in fields.iter() {
                    if let Err(e) = field.pattern.check(
                        /* allow type annotation: */ false,
                        /* is_inner_pattern: */ true,
                        session,
                    ) {
                        errors.extend(e);
                    }
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            PatternKind::TupleStruct { elements, .. } |
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                let mut errors = vec![];

                for element in elements.iter() {
                    if let Err(e) = element.check(
                        /* allow type annotation: */ false,
                        /* is_inner_pattern: */ true,
                        session,
                    ) {
                        errors.extend(e);
                    }
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            PatternKind::Range { lhs, rhs, op_span, is_inclusive } => {
                let mut errors = vec![];

                if *is_inclusive && rhs.is_none() {
                    errors.push(Error {
                        kind: ErrorKind::InclusiveRangeWithNoEnd,
                        spans: op_span.simple_error(),
                        ..Error::default()
                    });
                }

                // TODO: check range
                //       lhs and rhs can only be
                //       literal or dollar-ident
                if let Some(lhs) = lhs {
                    if let Err(e) = lhs.check_range_argument(true) {
                        errors.extend(e)
                    }

                    if let Err(e) = lhs.check(
                        /* allow type annotation: */ false,
                        /* is_inner_pattern: */ true,
                        session,
                    ) {
                        errors.extend(e);
                    }
                }

                if let Some(rhs) = rhs {
                    if let Err(e) = rhs.check_range_argument(false) {
                        errors.extend(e)
                    }

                    if let Err(e) = rhs.check(
                        /* allow type annotation: */ false,
                        /* is_inner_pattern: */ true,
                        session,
                    ) {
                        errors.extend(e);
                    }
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            PatternKind::Or { lhs, rhs, .. } => {
                let mut errors = vec![];

                if let Err(e) = lhs.check(
                    /* allow type annotation: */ false,
                    /* is_inner_pattern: */ true,
                    session,
                ) {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check(
                    /* allow type annotation: */ false,
                    /* is_inner_pattern: */ true,
                    session,
                ) {
                    errors.extend(e);
                }

                let mut lhs_name_binds = lhs.bound_names().iter().map(|(name, _)| *name).collect::<Vec<_>>();
                let mut rhs_name_binds = rhs.bound_names().iter().map(|(name, _)| *name).collect::<Vec<_>>();
                lhs_name_binds.sort();
                rhs_name_binds.sort();

                if lhs_name_binds != rhs_name_binds {
                    let mut lhs_name_binds = lhs_name_binds.iter().map(
                        |name| String::from_utf8_lossy(&unintern_string(*name, &session.intermediate_dir).unwrap().unwrap()).to_string()
                    ).collect::<Vec<_>>();
                    let mut rhs_name_binds = rhs_name_binds.iter().map(
                        |name| String::from_utf8_lossy(&unintern_string(*name, &session.intermediate_dir).unwrap().unwrap()).to_string()
                    ).collect::<Vec<_>>();
                    lhs_name_binds.sort();
                    rhs_name_binds.sort();

                    errors.push(Error {
                        kind: ErrorKind::DifferentNameBindingsInOrPattern,
                        spans: vec![
                            RenderableSpan {
                                span: lhs.error_span(),
                                auxiliary: false,
                                note: Some(format!(
                                    "This pattern binds {}: {}",
                                    if lhs_name_binds.len() == 1 { "a name" } else { "names" },
                                    comma_list_strs(&lhs_name_binds, "`", "`", "and"),
                                )),
                            },
                            RenderableSpan {
                                span: rhs.error_span(),
                                auxiliary: false,
                                note: Some(format!(
                                    "This pattern binds {}: {}",
                                    if rhs_name_binds.len() == 1 { "a name" } else { "names" },
                                    comma_list_strs(&rhs_name_binds, "`", "`", "and"),
                                )),
                            },
                        ],
                        note: Some(String::from("Names must be bound in all patterns.")),
                    });
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            // no type checks here!
            PatternKind::InfixOp { lhs, rhs, .. } => {
                let mut errors = vec![];

                if let Err(e) = lhs.check(false, true, session) {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check(false, true, session) {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
        }
    }
}
