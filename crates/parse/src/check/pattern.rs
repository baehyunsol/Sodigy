use crate::{FullPattern, Pattern};
use sodigy_error::{Error, ErrorKind};
use std::collections::hash_map::{Entry, HashMap};

impl FullPattern {
    pub fn check(
        &self,
        allow_type_annotation: bool,

        // If patterns are nested, we don't have to check name collisions
        // in the inner patterns. Also, we only type-check the outer-most pattern.
        is_inner_pattern: bool,
    ) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if !allow_type_annotation {
            if let Some(r#type) = &self.r#type {
                errors.push(Error {
                    kind: ErrorKind::CannotAnnotateType,
                    span: r#type.error_span(),
                    ..Error::default()
                });
            }
        }

        if !is_inner_pattern {
            let mut name_map = HashMap::new();

            for (name, name_span) in self.bound_names().iter() {
                match name_map.entry(*name) {
                    Entry::Occupied(e) => {
                        let prev_span = *e.get();
                        errors.push(Error {
                            kind: ErrorKind::NameCollision {
                                name: *name,
                            },
                            span: *name_span,
                            extra_span: Some(prev_span),
                            ..Error::default()
                        });
                    },
                    Entry::Vacant(e) => {
                        e.insert(*name_span);
                    },
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
                errors.push(Error {
                    kind: ErrorKind::RedundantNameBinding(*name, *id),
                    span: *name_span,
                    extra_span: Some(*span),
                    ..Error::default()
                });
            },
            _ => {},
        }

        if let Err(e) = self.pattern.check() {
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
    pub fn check(&self) -> Result<(), Vec<Error>> {
        match self {
            Pattern::Number { .. } |
            Pattern::Identifier { .. } |
            Pattern::Wildcard(_) => Ok(()),
            Pattern::Tuple { elements, .. } |
            Pattern::List { elements, .. } => {
                let mut errors = vec![];

                for element in elements.iter() {
                    if let Err(e) = element.check(
                        /* allow type annotation: */ false,
                        /* is_inner_pattern: */ true,
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
                        span: *op_span,
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
                            span: lhs.error_span(),
                            extra_message: Some(error_message.to_string()),
                            ..Error::default()
                        });
                    }
                }

                if let Some(rhs) = rhs {
                    let error_message = match rhs.as_ref() {
                        Pattern::Range { .. } => Some("A range-pattern cannot be an rhs of another range-pattern."),
                        Pattern::Or { .. } => Some("An or-pattern cannot be an rhs of a range-pattern."),
                        Pattern::Concat { .. } => Some("A concat-pattern cannot be an rhs of a range-pattern."),
                        _ => None,
                    };

                    if let Some(error_message) = error_message {
                        errors.push(Error {
                            kind: ErrorKind::AstPatternTypeError,
                            span: rhs.error_span(),
                            extra_message: Some(error_message.to_string()),
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

                if let Err(e) = lhs.check() {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check() {
                    errors.extend(e);
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

                if let Err(e) = lhs.check(false, true) {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check(false, true) {
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
                    Ok(PatternType::Integer)
                }

                else {
                    Ok(PatternType::Number)
                }
            },
            Pattern::Identifier { .. } |
            Pattern::Wildcard(_) => Ok(PatternType::NotSure),
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
            Pattern::List { elements, .. } => {
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
                                span: element.pattern.error_span(),
                                ..Error::default()
                            }]);
                        },
                    }
                }

                Ok(list_type)
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
                            span: lhs.as_ref().unwrap().error_span(),
                            extra_span: Some(rhs.as_ref().unwrap().error_span()),
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
            Pattern::Or { lhs, rhs, .. } => todo!(),
            Pattern::Concat { lhs, rhs, op_span } => {
                match (lhs.pattern.type_check(), rhs.pattern.type_check()) {
                    (Ok(lhs_type), Ok(rhs_type)) => match lhs_type.more_specific(&rhs_type) {
                        Ok(r#type) => Ok(r#type),
                        Err(()) => Err(vec![Error {
                            kind: ErrorKind::AstPatternTypeError,
                            span: *op_span,
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
    Integer,
    Number,
    String,
    BinaryString,
    Regex,
    Char,
    List(Box<PatternType>),
    Tuple(Vec<PatternType>),
}

impl PatternType {
    pub fn more_specific(&self, other: &PatternType) -> Result<PatternType, ()> {
        match (self, other) {
            (PatternType::NotSure, r#type) => Ok(r#type.clone()),
            (r#type, PatternType::NotSure) => Ok(r#type.clone()),
            (PatternType::Integer, PatternType::Integer) |
            (PatternType::Number, PatternType::Number) |
            (PatternType::String, PatternType::String) |
            (PatternType::BinaryString, PatternType::BinaryString) |
            (PatternType::Regex, PatternType::Regex) |
            (PatternType::Char, PatternType::Char) => Ok(self.clone()),
            (PatternType::List(type1), PatternType::List(type2)) => match type1.more_specific(type2) {
                Ok(r#type) => Ok(PatternType::List(Box::new(r#type))),
                Err(()) => Err(()),
            },
            (PatternType::Tuple(elements1), PatternType::Tuple(elements2)) => todo!(),
            _ => Err(()),
        }
    }
}
