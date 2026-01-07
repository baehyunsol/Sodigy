use crate::Type;
use sodigy_error::{Error, ErrorKind, Warning, WarningKind, comma_list_strs};
use sodigy_hir::{FuncOrigin, FuncPurity, LetOrigin};
use sodigy_mir::Session as MirSession;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::HashMap;

pub type TypeWarning = TypeError;

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

        // if `is_return`, `span` is a def_span of a function, and we're talking about the return type of the function.
        is_return: bool,
    },
    PartiallyInferedType {
        id: Option<InternedString>,
        span: Span,

        // if `is_return`, `r#type` is the return type of `id`.
        r#type: Type,
        is_return: bool,
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

    // Basically, it's just an `TypeError::UnexpectedType`, but I added a variant
    // for better error messages.
    UnexpectedPurity {
        expected_type: Type,
        expected_purity: FuncPurity,
        expected_span: Option<Span>,
        got_type: Type,
        got_purity: FuncPurity,
        got_span: Option<Span>,
    },

    // TODO: more fields
    CannotInferPolyGenericDef,
    CannotInferPolyGenericImpl,

    ImpureCallInPureContext {
        call_spans: Vec<Span>,
        keyword_span: Span,
        context: ExprContext,
    },
    NoImpureCallInImpureContext {  // warning by default
        impure_keyword_span: Span,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ExprContext {
    TopLevelLet,
    InlineLet,
    FuncDefaultValue,
    TopLevelFunc,
    InlineFunc,
    Lambda,
    TopLevelAssert,
}

impl From<LetOrigin> for ExprContext {
    fn from(o: LetOrigin) -> ExprContext {
        match o {
            LetOrigin::TopLevel => ExprContext::TopLevelLet,
            LetOrigin::Inline => ExprContext::InlineLet,
            LetOrigin::FuncDefaultValue => ExprContext::FuncDefaultValue,
            LetOrigin::Match => ExprContext::InlineLet,
        }
    }
}

impl From<FuncOrigin> for ExprContext {
    fn from(o: FuncOrigin) -> ExprContext {
        match o {
            FuncOrigin::TopLevel => ExprContext::TopLevelFunc,
            FuncOrigin::Inline => ExprContext::InlineFunc,
            FuncOrigin::Lambda => ExprContext::Lambda,
        }
    }
}

// TODO: naming sucks
#[derive(Clone, Debug)]
pub enum ErrorContext {
    AssertConditionBool,
    ShortCircuitAndBool,
    ShortCircuitOrBool,
    IfConditionBool,
    IfValueEqual,
    MatchScrutinee,
    MatchGuardBool,
    MatchArmEqual,
    InferTypeAnnotation,
    VerifyTypeAnnotation,
    ListElementEqual,
    FuncArgs,
    EqValueEqual,
    NeqValueEqual,
    OrPatternEqual,
    OrPatternNameBinding(InternedString),
    RangePatternEqual,

    // It infered the type of the same type var multiple times,
    // and got different result.
    InferedAgain { type_var: Type },

    Deep,

    // If there's nothing special about the context,
    // or the error kind tells everything about the context.
    None,
}

impl ErrorContext {
    pub fn note(&self, intermediate_dir: &str) -> Option<String> {
        match self {
            ErrorContext::AssertConditionBool => Some(String::from("An assertion must be a boolean.")),
            ErrorContext::ShortCircuitAndBool => Some(String::from("Lhs and rhs of `&&` operator must be booleans.")),
            ErrorContext::ShortCircuitOrBool => Some(String::from("Lhs and rhs of `||` operator must be booleans.")),
            ErrorContext::IfConditionBool => Some(String::from("A condition of an `if` expression must be a boolean.")),
            ErrorContext::IfValueEqual => Some(String::from("All branches of an `if` expression must have the same type.")),
            ErrorContext::MatchScrutinee => Some(String::from("A pattern of a match arm and the match's scrutinee must have the same type.")),
            ErrorContext::MatchGuardBool => Some(String::from("A guard of a match arm must be a boolean.")),
            ErrorContext::MatchArmEqual => Some(String::from("All arms of a `match` expression must have the same type.")),
            ErrorContext::InferTypeAnnotation => Some(String::from("There's an error while doing type-inference.")),
            ErrorContext::VerifyTypeAnnotation => Some(String::from("A value's type annotation and its actual type do not match.")),
            ErrorContext::ListElementEqual => Some(String::from("All elements of a list must have the same type.")),
            ErrorContext::FuncArgs => Some(String::from("Arguments of this function are incorrect.")),
            ErrorContext::EqValueEqual => Some(String::from("Lhs and rhs of `==` operator must have the same type.")),
            ErrorContext::NeqValueEqual => Some(String::from("Lhs and rhs of `!=` operator must have the same type.")),
            ErrorContext::OrPatternEqual => Some(String::from("Lhs and rhs of `|` pattern must have the same type.")),
            ErrorContext::OrPatternNameBinding(name) => Some(format!(
                "Name `{}` is bound multiple times in `|` pattern, but they have different types.",
                name.unintern_or_default(intermediate_dir),
            )),
            ErrorContext::RangePatternEqual => Some(String::from("Lhs and rhs of `..` pattern must have the same type.")),
            ErrorContext::InferedAgain { .. } => Some(String::from("I infered a type of the same value multiple times, and got different results.")),
            ErrorContext::Deep => Some(String::from("A contradiction is found while solving a chain of type-equations. There must be type error somewhere, but I can't find the exact location.")),
            ErrorContext::None => None,
        }
    }
}

pub fn type_error_to_general_error(error: &TypeError, session: &MirSession) -> Error {
    match error {
        TypeError::UnexpectedType {
            expected,
            expected_span,
            got,
            got_span,
            context,
        } => {
            let mut spans = vec![];
            let expected_type = session.render_type(expected);
            let got_type = session.render_type(got);

            if let ErrorContext::InferedAgain { type_var } = context {
                match type_var {
                    Type::Var { def_span, is_return } => {
                        spans.push(RenderableSpan {
                            span: *def_span,
                            auxiliary: false,
                            note: Some(format!(
                                "You didn't annotate the {}, so I tried to infer it. Some information says the type is `{}`, while another information says it's `{}`. Perhaps add a type annotation?",
                                if *is_return { "return type of thie function" } else { "type of this value" },
                                expected_type,
                                got_type,
                            )),
                        });
                    },
                    Type::GenericInstance { call, generic } => {
                        spans.push(RenderableSpan {
                            span: *call,
                            auxiliary: false,
                            note: Some(format!(
                                "This is a generic function, so I tried to figure out its type arguments. There's a problem with the type parameter `{}`. Some information says `{}`'s type is `{}`, while another information says it's `{}`.",
                                session.span_to_string(*generic).unwrap_or_else(|| String::from("???")),
                                expected_type,
                                session.span_to_string(*generic).unwrap_or_else(|| String::from("???")),
                                got_type,
                            )),
                        });
                    },
                    _ => unreachable!(),
                }

                if let Some(span) = *expected_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: false,
                        note: Some(format!("This information says the type is `{expected_type}`.")),
                    });
                }

                if let Some(span) = *got_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: false,
                        note: Some(format!("This information says the type is `{got_type}`.")),
                    });
                }
            }

            else {
                if let Some(span) = *expected_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: true,
                        note: Some(format!(
                            "The value should have type `{expected_type}`{}.",
                            if let ErrorContext::VerifyTypeAnnotation = context {
                                ", according to this type annotation"
                            } else {
                                ""
                            },
                        )),
                    });
                }

                if let Some(span) = *got_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: false,
                        note: Some(format!("This value is expected to have type `{expected_type}`, but has type `{got_type}`.")),
                    });
                }
            }

            Error {
                kind: ErrorKind::UnexpectedType {
                    expected: expected_type,
                    got: got_type,
                },
                spans,
                note: context.note(&session.intermediate_dir).map(|s| s.to_string()),
            }
        },
        TypeError::WrongNumberOfArguments {
            expected,
            got,
            given_keyword_arguments,
            func_span,
            arg_spans,
        } => {
            // With those information, we can guess which parameter is missing (or unnecessary)
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
        TypeError::CannotInferType { id, span, is_return } => Error {
            kind: ErrorKind::CannotInferType { id: *id, is_return: *is_return },
            spans: span.simple_error(),
            note: None,
        },
        TypeError::PartiallyInferedType {
            id,
            span,
            r#type,
            is_return,
        } => Error {
            kind: ErrorKind::PartiallyInferedType { id: *id, r#type: session.render_type(r#type), is_return: *is_return },
            spans: span.simple_error(),
            note: None,
        },
        TypeError::CannotInferGenericType { call, generic, func_def } |
        TypeError::PartiallyInferedGenericType { call, generic, func_def, .. } => {
            let generic_id = session.span_to_string(*generic);
            let spans = match (func_def.map(|def_span| session.func_shapes.get(&def_span)), &generic_id) {
                (Some(Some(func_shape)), Some(generic_id)) => vec![
                    RenderableSpan {
                        span: *call,
                        auxiliary: false,
                        note: Some(format!(
                            "This function has {} type parameter{} ({}), and I cannot infer the type of `{generic_id}`.",
                            func_shape.generics.len(),
                            if func_shape.generics.len() == 1 { "" } else { "s" },
                            comma_list_strs(
                                &func_shape.generics.iter().map(
                                    |generic_def| generic_def.name.unintern_or_default(&session.intermediate_dir)
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
                        r#type: session.render_type(r#type),
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
                        note: Some(format!("Cannot specialize `{}` here.", session.span_to_string(*poly_def).unwrap_or_else(|| String::from("????")))),
                    },
                    RenderableSpan {
                        span: *poly_def,
                        auxiliary: true,
                        note: Some(format!("`{}` is defined here.", session.span_to_string(*poly_def).unwrap_or_else(|| String::from("????")))),
                    },
                ],
                generics.iter().map(
                    |(span, r#type)| RenderableSpan {
                        span: *span,
                        auxiliary: true,
                        note: Some(format!("Type parameter `{}` is infered to be `{}`.", session.span_to_string(*span).unwrap_or_else(|| String::from("????")), session.render_type(r#type))),
                    }
                ).collect(),
            ].concat(),
            note: None,
        },
        TypeError::UnexpectedPurity {
            expected_type,
            expected_purity,
            expected_span,
            got_type,
            got_purity,
            got_span,
        } => {
            let mut spans = vec![];
            let expected_type = session.render_type(expected_type);
            let got_type = session.render_type(got_type);

            if let Some(span) = *expected_span {
                let note = match expected_purity {
                    FuncPurity::Pure => "It expects a pure function.",
                    FuncPurity::Impure => "It expects an impure function.",
                    FuncPurity::Both => unreachable!(),
                }.to_string();

                spans.push(RenderableSpan {
                    span,
                    auxiliary: true,
                    note: Some(note),
                });
            }

            if let Some(span) = *got_span {
                let note = match got_purity {
                    FuncPurity::Pure => "This is a pure function.",
                    FuncPurity::Impure => "This is an impure function.",
                    FuncPurity::Both => "I'm not sure whether it's pure or not.",
                }.to_string();

                spans.push(RenderableSpan {
                    span,
                    auxiliary: false,
                    note: Some(note),
                });
            }

            let note = match (expected_purity, got_purity) {
                (ex, FuncPurity::Both) => Some(format!(
                    "If you're sure that this is {}, add a type annotation. Be careful that `Fn` is for 'pure or impure' functions, you have to use `PureFn` or `ImpureFn` to be clear.",
                    match ex { FuncPurity::Pure => "pure", FuncPurity::Impure => "impure", FuncPurity::Both => unreachable!() },
                )),
                _ => None,
            };

            Error {
                kind: ErrorKind::UnexpectedType {
                    expected: expected_type,
                    got: got_type,
                },
                spans,
                note,
            }
        },
        TypeError::ImpureCallInPureContext { call_spans, keyword_span, context } => {
            let mut spans = vec![];
            let (keyword_note, error_note) = match context {
                ExprContext::TopLevelLet => (Some("This is a top-level `let` statement, and it has to be pure. If you want to do impure stuffs, define an impure function."), None),
                ExprContext::InlineLet => unreachable!(),
                ExprContext::FuncDefaultValue => (None, Some("You can't call impure functions when initializing a default value.")),
                ExprContext::TopLevelFunc | ExprContext::InlineFunc => (
                    Some("A function is pure by default. If you want to define an impure function, add `impure` keyword before the `fn` keyword."),
                    None,
                ),
                ExprContext::Lambda => (Some("A lambda function is pure by default. If you want the lambda to be impure, add `impure` keyword before the backslash."), None),
                ExprContext::TopLevelAssert => (Some("You can't call impure functions when asserting something."), None),
            };
            let (keyword_note, error_note) = (keyword_note.map(|s| s.to_string()), error_note.map(|s| s.to_string()));

            spans.push(RenderableSpan {
                span: *keyword_span,
                auxiliary: true,
                note: keyword_note,
            });

            for call_span in call_spans.iter() {
                spans.push(RenderableSpan {
                    span: *call_span,
                    auxiliary: false,
                    note: Some(String::from("You're calling an impure function here.")),
                });
            }

            Error {
                kind: ErrorKind::ImpureCallInPureContext,
                spans,
                note: error_note,
            }
        },

        // This is a warning, so don't expect `init_span_string_map()`!
        TypeWarning::NoImpureCallInImpureContext { impure_keyword_span } => Warning {
            kind: WarningKind::NoImpureCallInImpureContext,
            spans: vec![RenderableSpan {
                span: *impure_keyword_span,
                auxiliary: false,
                note: Some(String::from("This `impure` keyword makes this function impure.")),
            }],
            note: None,
        },
        _ => panic!("TODO: {error:?}"),
    }
}
