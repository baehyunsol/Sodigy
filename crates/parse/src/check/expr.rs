use super::check_call_args;
use crate::Expr;
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Expr {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        match self {
            Expr::Identifier { .. } |
            Expr::Number { .. } |
            Expr::String { .. } => Ok(()),
            Expr::If(r#if) => r#if.check(),
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
            Expr::Tuple { elements, .. } => {
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
                let mut span_by_name: HashMap<InternedString, Span> = HashMap::new();
                let mut must_have_default_value = false;

                for arg in args.iter() {
                    if must_have_default_value && arg.default_value.is_none() {
                        errors.push(Error {
                            kind: ErrorKind::NonDefaultValueAfterDefaultValue,
                            span: arg.name_span,
                            ..Error::default()
                        });
                    }

                    if let Err(e) = arg.check() {
                        errors.extend(e);
                    }

                    if arg.default_value.is_some() {
                        must_have_default_value = true;
                    }

                    match span_by_name.entry(arg.name) {
                        Entry::Occupied(e) => {
                            errors.push(Error {
                                kind: ErrorKind::NameCollision {
                                    name: arg.name,
                                },
                                span: arg.name_span,
                                extra_span: Some(*e.get()),
                                ..Error::default()
                            });
                        },
                        Entry::Vacant(e) => {
                            e.insert(arg.name_span);
                        },
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
            _ => panic!("TODO: {self:?}"),
        }
    }
}
