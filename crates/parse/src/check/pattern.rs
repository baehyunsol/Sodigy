use crate::{Pattern, PatternKind, PatternValueKind};
use sodigy_error::{Error, ErrorKind, NameCollisionKind, comma_list_strs};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Pattern {
    pub fn check(
        &self,

        // If patterns are nested, we don't have to check name collisions
        // in the inner patterns. Also, we only type-check the outer-most pattern.
        is_inner_pattern: bool,
        intermediate_dir: &str,
    ) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

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
                            name: *name,
                            kind: NameCollisionKind::Pattern,
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

        if let Err(e) = self.kind.check(intermediate_dir) {
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
            PatternKind::Number { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } => Ok(()),
            PatternKind::InfixOp { kind, .. } => match kind {
                PatternValueKind::Constant => Ok(()),
                PatternValueKind::DollarIdent | PatternValueKind::Ident => {
                    let note = match kind {
                        PatternValueKind::DollarIdent => "Dollar-identifiers cannot be an end of a range.",
                        PatternValueKind::Ident => "You cannot bind a name to an end of a range.",
                        _ => unreachable!(),
                    };

                    Err(vec![Error {
                        kind: ErrorKind::InvalidRangePattern,
                        spans: self.error_span_wide().simple_error(),
                        note: Some(note.to_string()),
                    }])
                },
            },

            _ => Err(vec![Error {
                kind: ErrorKind::InvalidRangePattern,
                spans: self.error_span_wide().simple_error(),
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
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        match self {
            PatternKind::Number { .. } |
            PatternKind::String { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } |
            PatternKind::Ident { .. } |
            PatternKind::Path(_) |
            PatternKind::Wildcard(_) |
            PatternKind::PipelineData(_) |
            PatternKind::DollarIdent { .. } => Ok(()),
            PatternKind::Regex { .. } => todo!(),
            PatternKind::Struct { fields, .. } => {
                // There maybe name collisions in the fields, but AST doesn't care about that.
                let mut errors = vec![];

                for field in fields.iter() {
                    if let Err(e) = field.pattern.check(/* is_inner_pattern: */ true, intermediate_dir) {
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
                    if let Err(e) = element.check(/* is_inner_pattern: */ true, intermediate_dir) {
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
                        note: None,
                    });
                }

                if let Some(lhs) = lhs {
                    if let Err(e) = lhs.check_range_argument(true) {
                        errors.extend(e)
                    }

                    if let Err(e) = lhs.check(/* is_inner_pattern: */ true, intermediate_dir) {
                        errors.extend(e);
                    }
                }

                if let Some(rhs) = rhs {
                    if let Err(e) = rhs.check_range_argument(false) {
                        errors.extend(e)
                    }

                    if let Err(e) = rhs.check(/* is_inner_pattern: */ true, intermediate_dir) {
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

                if let Err(e) = lhs.check(/* is_inner_pattern: */ true, intermediate_dir) {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check(/* is_inner_pattern: */ true, intermediate_dir) {
                    errors.extend(e);
                }

                let mut lhs_name_binds = lhs.bound_names().iter().map(|(name, _)| *name).collect::<Vec<_>>();
                let mut rhs_name_binds = rhs.bound_names().iter().map(|(name, _)| *name).collect::<Vec<_>>();
                lhs_name_binds.sort();
                rhs_name_binds.sort();

                if lhs_name_binds != rhs_name_binds {
                    let mut lhs_name_binds = lhs_name_binds.iter().map(
                        |name| name.unintern_or_default(intermediate_dir)
                    ).collect::<Vec<_>>();
                    let mut rhs_name_binds = rhs_name_binds.iter().map(
                        |name| name.unintern_or_default(intermediate_dir)
                    ).collect::<Vec<_>>();
                    lhs_name_binds.sort();
                    rhs_name_binds.sort();

                    errors.push(Error {
                        kind: ErrorKind::DifferentNameBindingsInOrPattern,
                        spans: vec![
                            RenderableSpan {
                                span: lhs.error_span_wide(),
                                auxiliary: false,
                                note: Some(format!(
                                    "This pattern binds {}: {}",
                                    if lhs_name_binds.len() == 1 { "a name" } else { "names" },
                                    comma_list_strs(&lhs_name_binds, "`", "`", "and"),
                                )),
                            },
                            RenderableSpan {
                                span: rhs.error_span_wide(),
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

                if let Err(e) = lhs.check(true, intermediate_dir) {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check(true, intermediate_dir) {
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
