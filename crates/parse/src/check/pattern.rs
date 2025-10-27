use crate::{FullPattern, Pattern, Session};
use sodigy_error::{Error, ErrorKind, comma_list_strs};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, unintern_string};
use std::collections::hash_map::{Entry, HashMap};

impl FullPattern {
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

            if let Err(e) = self.pattern.type_check() {
                errors.extend(e);
            }
        }

        match self {
            FullPattern {
                name: Some(name),
                name_span: Some(name_span),
                r#type: _,
                pattern: Pattern::Identifier { id, span },
            } => {
                let note1 = format!(
                    "Name `{}` is bound to the pattern.",
                    String::from_utf8_lossy(&unintern_string(*name, &session.intermediate_dir).unwrap().unwrap()).to_string(),
                );
                let note2 = format!(
                    "Name `{}` is bound to the pattern.",
                    String::from_utf8_lossy(&unintern_string(*id, &session.intermediate_dir).unwrap().unwrap()).to_string(),
                );

                errors.push(Error {
                    kind: ErrorKind::RedundantNameBinding(*name, *id),
                    spans: vec![
                        RenderableSpan {
                            span: *name_span,
                            auxiliary: false,
                            note: Some(note1),
                        },
                        RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: Some(note2),
                        },
                    ],
                    ..Error::default()
                });
            },
            _ => {},
        }

        if let Err(e) = self.pattern.check(session) {
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

impl Pattern {
    pub fn check(&self, session: &Session) -> Result<(), Vec<Error>> {
        match self {
            Pattern::Number { .. } |
            Pattern::Identifier { .. } |
            Pattern::Path(_) |
            Pattern::Wildcard(_) => Ok(()),
            Pattern::Struct { fields, .. } => {
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
            Pattern::TupleStruct { elements, .. } |
            Pattern::Tuple { elements, .. } |
            Pattern::List { elements, .. } => {
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
            Pattern::Range { lhs, rhs, op_span, is_inclusive } => {
                let mut errors = vec![];

                if *is_inclusive && rhs.is_none() {
                    errors.push(Error {
                        kind: ErrorKind::InclusiveRangeWithNoEnd,
                        spans: op_span.simple_error(),
                        ..Error::default()
                    });
                }

                // `Pattern::type_check` can't catch this
                if let Some(lhs) = lhs {
                    let error_message = match lhs.as_ref() {
                        Pattern::Range { .. } => Some("A range-pattern cannot be an lhs of another range-pattern."),
                        Pattern::Or { .. } => Some("An or-pattern cannot be an lhs of a range-pattern."),
                        Pattern::Concat { .. } => Some("A concat-pattern cannot be an lhs of a range-pattern."),
                        _ => None,
                    };

                    if let Some(error_message) = error_message {
                        errors.push(Error {
                            kind: ErrorKind::AstPatternTypeError,
                            spans: lhs.error_span().simple_error(),
                            note: Some(error_message.to_string()),
                            ..Error::default()
                        });
                    }
                }

                if let Some(rhs) = rhs {
                    let note = match rhs.as_ref() {
                        Pattern::Range { .. } => Some("A range-pattern cannot be an rhs of another range-pattern."),
                        Pattern::Or { .. } => Some("An or-pattern cannot be an rhs of a range-pattern."),
                        Pattern::Concat { .. } => Some("A concat-pattern cannot be an rhs of a range-pattern."),
                        _ => None,
                    };

                    if let Some(note) = note {
                        errors.push(Error {
                            kind: ErrorKind::AstPatternTypeError,
                            spans: rhs.error_span().simple_error(),
                            note: Some(note.to_string()),
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
            },
            Pattern::Or { lhs, rhs, .. } => {
                let mut errors = vec![];

                if let Err(e) = lhs.check(session) {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check(session) {
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
            Pattern::Concat { lhs, rhs, .. } => {
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

    fn type_check(&self) -> Result<PatternType, Vec<Error>> {
        match self {
            Pattern::Number { n, .. } => {
                if n.is_integer {
                    Ok(PatternType::Int)
                }

                else {
                    Ok(PatternType::Number)
                }
            },
            Pattern::Identifier { .. } |
            Pattern::Wildcard(_) |
            Pattern::Path(_) => Ok(PatternType::NotSure),
            Pattern::Struct { fields, .. } => {
                let mut errors = vec![];

                for field in fields.iter() {
                    if let Err(e) = field.pattern.pattern.type_check() {
                        errors.extend(e);
                    }
                }

                if errors.is_empty() {
                    Ok(PatternType::NotSure)
                }

                else {
                    Err(errors)
                }
            },
            Pattern::TupleStruct { elements, .. } |
            Pattern::Tuple { elements, .. } => {
                let mut types = Vec::with_capacity(elements.len());
                let mut errors = vec![];

                for element in elements.iter() {
                    match element.pattern.type_check() {
                        Ok(r#type) => {
                            types.push(r#type);
                        },
                        Err(e) => {
                            errors.extend(e);
                        },
                    }
                }

                if errors.is_empty() {
                    Ok(PatternType::Tuple(types))
                }

                else {
                    Err(errors)
                }
            },
            Pattern::List { elements, group_span } => {
                let mut list_type = PatternType::NotSure;

                for element in elements.iter() {
                    let element_type = element.pattern.type_check()?;

                    match list_type.more_specific(&element_type) {
                        Ok(r#type) => {
                            list_type = r#type;
                        },
                        Err(()) => {
                            return Err(vec![Error {
                                kind: ErrorKind::AstPatternTypeError,
                                spans: vec![
                                    RenderableSpan {
                                        span: element.pattern.error_span(),
                                        auxiliary: false,
                                        note: Some(format!("This has type `{}`.", element_type.render())),
                                    },
                                    RenderableSpan {
                                        span: group_span.begin(),
                                        auxiliary: true,
                                        note: Some(format!("This has type `{}`.", PatternType::List(Box::new(list_type)).render())),
                                    },
                                ],
                                note: None,
                            }]);
                        },
                    }
                }

                Ok(PatternType::List(Box::new(list_type)))
            },
            Pattern::Range { lhs, rhs, .. } => {
                match (
                    lhs.as_ref().map(|lhs| lhs.type_check()),
                    rhs.as_ref().map(|rhs| rhs.type_check()),
                ) {
                    (Some(Ok(lhs_type)), Some(Ok(rhs_type))) => match lhs_type.more_specific(&rhs_type) {
                        Ok(r#type) => Ok(r#type),
                        Err(()) => Err(vec![Error {
                            kind: ErrorKind::AstPatternTypeError,
                            spans: vec![
                                RenderableSpan {
                                    span: lhs.as_ref().unwrap().error_span(),
                                    auxiliary: false,
                                    note: Some(format!("This has type `{}`.", lhs_type.render())),
                                },
                                RenderableSpan {
                                    span: rhs.as_ref().unwrap().error_span(),
                                    auxiliary: false,
                                    note: Some(format!("This has type `{}`.", rhs_type.render())),
                                },
                            ],
                            ..Error::default()
                        }]),
                    },
                    (Some(Ok(r#type)), None) |
                    (None, Some(Ok(r#type))) => Ok(r#type),
                    (Some(Err(lhs_error)), Some(Err(rhs_error))) => Err(vec![lhs_error, rhs_error].concat()),
                    (Some(Err(error)), _) |
                    (_, Some(Err(error))) => Err(error),

                    // The parser guarantees that it's unreachable.
                    (None, None) => Ok(PatternType::NotSure),
                }
            },
            Pattern::Or { lhs, rhs, .. } => {
                match (lhs.type_check(), rhs.type_check()) {
                    (Ok(lhs_type), Ok(rhs_type)) => match lhs_type.more_specific(&rhs_type) {
                        Ok(r#type) => Ok(r#type),
                        Err(()) => Err(vec![Error {
                            kind: ErrorKind::AstPatternTypeError,
                            spans: vec![
                                RenderableSpan {
                                    span: lhs.error_span(),
                                    auxiliary: false,
                                    note: Some(format!("This has type `{}`.", lhs_type.render())),
                                },
                                RenderableSpan {
                                    span: rhs.error_span(),
                                    auxiliary: false,
                                    note: Some(format!("This has type `{}`.", rhs_type.render())),
                                },
                            ],
                            ..Error::default()
                        }]),
                    },
                    (Err(lhs_error), Err(rhs_error)) => Err(vec![lhs_error, rhs_error].concat()),
                    (Err(e), _) | (_, Err(e)) => Err(e),
                }
            },
            Pattern::Concat { lhs, rhs, .. } => {
                match (lhs.pattern.type_check(), rhs.pattern.type_check()) {
                    (Ok(lhs_type), Ok(rhs_type)) => match lhs_type.more_specific(&rhs_type) {
                        Ok(r#type) => Ok(r#type),
                        Err(()) => Err(vec![Error {
                            kind: ErrorKind::AstPatternTypeError,
                            spans: vec![
                                RenderableSpan {
                                    span: lhs.error_span(),
                                    auxiliary: false,
                                    note: Some(format!("This has type `{}`.", lhs_type.render())),
                                },
                                RenderableSpan {
                                    span: rhs.error_span(),
                                    auxiliary: false,
                                    note: Some(format!("This has type `{}`.", rhs_type.render())),
                                },
                            ],
                            ..Error::default()
                        }]),
                    },
                    (Err(lhs_error), Err(rhs_error)) => Err(vec![lhs_error, rhs_error].concat()),
                    (Err(e), _) | (_, Err(e)) => Err(e),
                }
            },
        }
    }
}

// We can do basic type-checks in AST level.
// For example, the AST can tell `0..""` is a type-error.
// Full type-check will be done by MIR.
#[derive(Clone, Debug)]
enum PatternType {
    NotSure,  // e.g. identifier, wildcard, ...
    Int,
    Number,
    String,
    Bytes,
    Regex,
    Char,
    List(Box<PatternType>),
    Tuple(Vec<PatternType>),
}

impl PatternType {
    // It's kinda type-check + subtyping.
    // If the two types are the same, it returns the type.
    // If type A is a subtype of type B, it returns B.
    // Otherwise, it returns Err.
    pub fn more_specific(&self, other: &PatternType) -> Result<PatternType, ()> {
        match (self, other) {
            (PatternType::NotSure, r#type) => Ok(r#type.clone()),
            (r#type, PatternType::NotSure) => Ok(r#type.clone()),
            (PatternType::Int, PatternType::Int) |
            (PatternType::Number, PatternType::Number) |
            (PatternType::String, PatternType::String) |
            (PatternType::Bytes, PatternType::Bytes) |
            (PatternType::Regex, PatternType::Regex) |
            (PatternType::Char, PatternType::Char) => Ok(self.clone()),
            (PatternType::List(type1), PatternType::List(type2)) => match type1.more_specific(type2) {
                Ok(r#type) => Ok(PatternType::List(Box::new(r#type))),
                Err(()) => Err(()),
            },
            (PatternType::Tuple(elements1), PatternType::Tuple(elements2)) => {
                if elements1.len() != elements2.len() {
                    Err(())
                }

                else {
                    let mut elements = Vec::with_capacity(elements1.len());

                    for i in 0..elements1.len() {
                        elements.push(elements1[i].more_specific(&elements2[i])?);
                    }

                    Ok(PatternType::Tuple(elements))
                }
            },
            _ => Err(()),
        }
    }

    // for error messages
    pub fn render(&self) -> String {
        match self {
            PatternType::NotSure => String::from("_"),
            PatternType::Int => String::from("Int"),
            PatternType::Number => String::from("Number"),
            PatternType::String => String::from("String"),
            PatternType::Bytes => String::from("Bytes"),

            // TODO: do I need another annotation?
            PatternType::Regex => String::from("String"),

            PatternType::Char => String::from("Char"),
            PatternType::List(element) => format!("[{}]", element.render()),
            PatternType::Tuple(elements) => format!(
                "({})",
                elements.iter().map(
                    |e| e.render()
                ).collect::<Vec<_>>().join(", "),
            ),
        }
    }
}
