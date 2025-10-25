use super::check_call_args;
use crate::Expr;
use sodigy_error::{Error, ErrorKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Expr {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        match self {
            Expr::Identifier { .. } |
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } => Ok(()),
            Expr::If(r#if) => r#if.check(),
            Expr::Match(r#match) => r#match.check(),
            Expr::Block(block) => block.check(false /* top_level */),
            Expr::Call { func, args } => {
                let mut errors = vec![];

                if let Err(e) = func.check() {
                    errors.extend(e);
                }

                if let Err(e) = check_call_args(args) {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            Expr::Tuple { elements, .. } |
            Expr::List { elements, .. } => {
                let mut errors = vec![];

                for element in elements.iter() {
                    if let Err(e) = element.check() {
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
            Expr::StructInit {
                r#struct,
                fields,
                ..
            } => {
                let mut errors = vec![];

                if let Err(e) = r#struct.check() {
                    errors.extend(e);
                }

                for field in fields.iter() {
                    if let Err(e) = field.value.check() {
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
            Expr::Path { lhs, .. } => {
                let mut errors = vec![];

                if let Err(e) = lhs.check() {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            Expr::Lambda { args, r#type, value, .. } => {
                let mut errors = vec![];
                let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

                // for error messages
                let mut span_of_arg_with_default_value = None;

                for arg in args.iter() {
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

                    if let Err(e) = arg.check() {
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
                }

                if let Some(r#type) = r#type.as_ref() {
                    if let Err(e) = r#type.check() {
                        errors.extend(e);
                    }
                }

                if let Err(e) = value.check() {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            Expr::InfixOp { lhs, rhs, .. } |
            Expr::FieldModifier { lhs, rhs, .. } => {
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
        }
    }
}
