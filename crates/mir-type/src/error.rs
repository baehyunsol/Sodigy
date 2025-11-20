use crate::Type;
use sodigy_error::{Error, ErrorKind, comma_list_strs};
use sodigy_mir::Session as MirSession;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, unintern_string};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum TypeError {
    UnexpectedType {
        expected: Type,
        expected_span: Option<Span>,
        got: Type,
        got_span: Option<Span>,
        context: ErrorContext,
    },
    // Since it's a very common error, the compiler tries to
    // give an as helpful error message as possible
    WrongNumberOfArguments {
        expected: Vec<Type>,
        got: Vec<Type>,

        // It has type `Vec<(keyword: InternedString, n: usize)>` where
        // `n`th argument of `got` has keyword `keyword`.
        given_keyword_arguments: Vec<(InternedString, usize)>,

        func_span: Span,
        arg_spans: Vec<Span>,
    },
    CannotInferType {
        id: Option<InternedString>,
        span: Span,
    },
    PartiallyInferedType {
        id: Option<InternedString>,
        span: Span,
        r#type: Type,
    },
    CannotInferGenericType {
        call: Span,
        generic: Span,
        func_def: Option<Span>,
    },
    PartiallyInferedGenericType {
        call: Span,
        generic: Span,
        func_def: Option<Span>,
        r#type: Type,
    },
    NotCallable {
        r#type: Type,
        func_span: Span,
    },
    CannotSpecializePolyGeneric {
        call: Span,
        poly_def: Span,
        generics: HashMap<Span, Type>,
        num_candidates: usize,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorContext {
    AssertConditionBool,
    ShortCircuitAndBool,
    ShortCircuitOrBool,
    IfConditionBool,
    IfValueEqual,
    InferTypeAnnotation,
    VerifyTypeAnnotation,
    ListElementEqual,
    FuncArgs,
    EqValueEqual,
    NeqValueEqual,
    Deep,

    // If there's nothing special about the context,
    // or the error kind tells everything about the context.
    None,
}

impl ErrorContext {
    pub fn note(&self) -> Option<&'static str> {
        match self {
            ErrorContext::AssertConditionBool => Some("An assertion must be a boolean."),
            ErrorContext::ShortCircuitAndBool => Some("Lhs and rhs of `&&` operator must be booleans."),
            ErrorContext::ShortCircuitOrBool => Some("Lhs and rhs of `||` operator must be booleans."),
            ErrorContext::IfConditionBool => Some("A condition of an `if` expression must be a boolean."),
            ErrorContext::IfValueEqual => Some("All branches of an `if` expression must have the same type."),
            ErrorContext::InferTypeAnnotation => Some("There's an error while doing type-inference."),
            ErrorContext::VerifyTypeAnnotation => Some("A type annotation and its actual type do not match."),
            ErrorContext::ListElementEqual => Some("All elements of a list must have the same type."),
            ErrorContext::FuncArgs => Some("Arguments of this function are incorrect."),
            ErrorContext::EqValueEqual => Some("Lhs and rhs of `==` operator must have the same type."),
            ErrorContext::NeqValueEqual => Some("Lhs and rhs of `!=` operator must have the same type."),
            ErrorContext::Deep => Some("A contradiction is found while solving a chain of type-equations. There must be type error somewhere, but I can't find the exact location."),
            ErrorContext::None => None,
        }
    }
}

pub trait RenderTypeError {
    fn type_error_to_general_error(&self, error: &TypeError) -> Error;
    fn render_type(&self, r#type: &Type) -> String;
    fn span_to_string(&self, span: Span) -> Option<String>;
}

impl RenderTypeError for MirSession {
    fn type_error_to_general_error(&self, error: &TypeError) -> Error {
        match error {
            TypeError::UnexpectedType {
                expected,
                expected_span,
                got,
                got_span,
                context,
            } => {
                let mut spans = vec![];
                let expected_type = self.render_type(expected);
                let got_type = self.render_type(got);

                if let Some(span) = *expected_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: true,
                        note: Some(format!("This value has type `{expected_type}`.")),
                    });
                }

                if let Some(span) = *got_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: false,
                        note: Some(format!("This value is expected to have type `{expected_type}`, but has type `{got_type}`.")),
                    });
                }

                Error {
                    kind: ErrorKind::UnexpectedType {
                        expected: expected_type,
                        got: got_type,
                    },
                    spans,
                    note: context.note().map(|s| s.to_string()),
                }
            },
            TypeError::WrongNumberOfArguments {
                expected,
                got,
                given_keyword_arguments,
                func_span,
                arg_spans,
            } => {
                // With those information, we can guess which argument is missing (or unnecessary)
                //
                // 1. If the user has used keyword arguments, that cannot be a missing or an unnecessary argument.
                //    We have to filter them out.
                // 2. TODO: we have to check whether an argument is provided by the user or a default value.
                //    If it's a default value, that cannot be a missing or an unnecessary argument. We have to filter them out.
                // 3. try to substitute type variables in `expected` and `got`.
                //    - those fields are captured when this error's created
                //    - there might be updates in the type variables
                // 4. TODO: then what?
                todo!()
            },
            TypeError::CannotInferType { id, span } => Error {
                kind: ErrorKind::CannotInferType { id: *id },
                spans: span.simple_error(),
                note: None,
            },
            TypeError::PartiallyInferedType {
                id,
                span,
                r#type,
            } => Error {
                kind: ErrorKind::PartiallyInferedType { id: *id, r#type: self.render_type(r#type) },
                spans: span.simple_error(),
                note: None,
            },
            TypeError::CannotInferGenericType { call, generic, func_def } |
            TypeError::PartiallyInferedGenericType { call, generic, func_def, .. } => {
                let generic_id = self.span_to_string(*generic);
                let spans = match (func_def.map(|def_span| self.func_shapes.get(&def_span)), &generic_id) {
                    (Some(Some((_, generic_defs))), Some(generic_id)) => vec![
                        RenderableSpan {
                            span: *call,
                            auxiliary: false,
                            note: Some(format!(
                                "This function has {} type parameter{} ({}), and I cannot infer the type of `{generic_id}`.",
                                generic_defs.len(),
                                if generic_defs.len() == 1 { "" } else { "s" },
                                comma_list_strs(
                                    &generic_defs.iter().map(
                                        |generic_def| String::from_utf8_lossy(&unintern_string(generic_def.name, &self.intermediate_dir).unwrap().unwrap()).to_string()
                                    ).collect::<Vec<_>>(),
                                    "`",
                                    "`",
                                    "and",
                                ),
                            )),
                        },
                        RenderableSpan {
                            span: *generic,
                            auxiliary: true,
                            note: Some(format!("Type parameter `{generic_id}` is defined here.")),
                        },
                    ],
                    _ => call.simple_error(),
                };

                match error {
                    TypeError::CannotInferGenericType { .. } => Error {
                        kind: ErrorKind::CannotInferGenericType { id: generic_id },
                        spans,
                        note: None,
                    },
                    TypeError::PartiallyInferedGenericType { r#type, .. } => Error {
                        kind: ErrorKind::PartiallyInferedGenericType {
                            id: generic_id,
                            r#type: self.render_type(r#type),
                        },
                        spans,
                        note: None,
                    },
                    _ => unreachable!(),
                }
            },
            // TODO: based on the poly's def_span, I want it to throw
            //       `CannotApplyInfixOp` or so.
            TypeError::CannotSpecializePolyGeneric {
                call,
                poly_def,
                generics,
                num_candidates,
            } => Error {
                kind: ErrorKind::CannotSpecializePolyGeneric {
                    num_candidates: *num_candidates,
                },
                spans: vec![
                    vec![
                        RenderableSpan {
                            span: *call,
                            auxiliary: false,
                            note: Some(format!("Cannot specialize `{}` here.", self.span_to_string(*poly_def).unwrap_or_else(|| String::from("????")))),
                        },
                        RenderableSpan {
                            span: *poly_def,
                            auxiliary: true,
                            note: Some(format!("`{}` is defined here.", self.span_to_string(*poly_def).unwrap_or_else(|| String::from("????")))),
                        },
                    ],
                    generics.iter().map(
                        |(span, r#type)| RenderableSpan {
                            span: *span,
                            auxiliary: true,
                            note: Some(format!("Type parameter `{}` is infered to be `{}`.", self.span_to_string(*span).unwrap_or_else(|| String::from("????")), self.render_type(r#type))),
                        }
                    ).collect(),
                ].concat(),
                note: None,
            },
            _ => todo!(),
        }
    }

    fn render_type(&self, r#type: &Type) -> String {
        match r#type {
            Type::Static(span) | Type::GenericDef(span) => self.span_to_string(*span).unwrap_or_else(|| String::from("????")),
            Type::Unit(_) => String::from("()"),
            Type::Param {
                r#type,
                args,
                ..
            } if matches!(r#type.as_ref(), Type::Unit(_)) => format!(
                "({}{})",
                args.iter().map(
                    |arg| self.render_type(arg)
                ).collect::<Vec<_>>().join(", "),
                if args.len() == 1 { "," } else { "" },
            ),
            // TODO: alias `List<T>` to `[T]`?
            Type::Param { r#type, args, .. } => format!(
                "{}<{}>",
                self.render_type(r#type),
                args.iter().map(
                    |arg| self.render_type(arg)
                ).collect::<Vec<_>>().join(", "),
            ),
            Type::Func { args, r#return, .. } => format!(
                "Fn({}) -> {}",
                args.iter().map(
                    |arg| self.render_type(arg)
                ).collect::<Vec<_>>().join(", "),
                self.render_type(r#return.as_ref()),
            ),
            Type::Var { .. } | Type::GenericInstance { .. } => String::from("_"),
            Type::Never { .. } => String::from("!"),
        }
    }

    fn span_to_string(&self, span: Span) -> Option<String> {
        match span {
            Span::Prelude(p) => match unintern_string(p, &self.intermediate_dir) {
                Ok(Some(p)) => Some(String::from_utf8_lossy(&p).to_string()),
                _ => None,
            },
            Span::Range { .. } => match self.span_string_map.as_ref().map(|map| map.get(&span)) {
                Some(Some(s)) => match unintern_string(*s, &self.intermediate_dir) {
                    Ok(Some(s)) => Some(String::from_utf8_lossy(&s).to_string()),
                    _ => None,
                },
                _ => None,
            },
            Span::None => None,
            _ => todo!(),
        }
    }
}
