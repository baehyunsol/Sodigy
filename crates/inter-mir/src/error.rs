use crate::{Session, Type};
use sodigy_error::{
    Error,
    ErrorKind,
    FuncEffect,
    ParamIndex,
    TypeVarInfo,
    Warning,
    WarningKind,
    comma_list_strs,
    to_ordinal,
};
use sodigy_hir::{FuncOrigin, LetOrigin};
use sodigy_mir::{render_type, span_to_string};
use sodigy_parse::Field;
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
    WrongNumberOfArgs {
        expected: Vec<Type>,
        got: Vec<Type>,

        // It has type `Vec<(keyword: InternedString, n: usize)>` where
        // `n`th argument of `got` has keyword `keyword`.
        given_keyword_args: Vec<(InternedString, usize)>,

        call: Span,
        def: Option<Span>,
        arg_spans: Vec<Span>,
    },
    WrongNumberOfGenericArgs {
        expected: usize,
        got: usize,
        param_group_span: Span,
        arg_group_span: Span,
    },
    UnnecessaryGenericArgs {
        def_span: Span,
        num_generic_args: usize,
        r#type: Type,
        call_span: Option<Span>,
    },
    MissingGenericArgs {
        def_span: Span,
        num_generic_params: usize,
        r#type: Type,
        call_span: Option<Span>,
    },
    CannotInferType {
        info: Option<TypeVarInfo>,
        span: Span,

        // if `is_return`, `span` is a def_span of a function, and we're talking about the return type of the function.
        is_return: bool,
    },
    PartiallyInferedType {
        info: Option<TypeVarInfo>,
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
    UnknownField {
        r#type: Type,
        field: Field,
    },
    CannotUpdateAssociatedFunc {
        r#type: Type,
        name: InternedString,
        name_span: Span,
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
    UnexpectedEffect {
        expected_type: Type,
        expected_effect: FuncEffect,
        expected_span: Option<Span>,
        got_type: Type,
        got_effect: FuncEffect,
        got_span: Option<Span>,
    },

    CannotInferPolyGenericParam {
        poly_span: Span,
        param_index: ParamIndex,
    },
    CannotInferPolyGenericImpl {
        poly_span: Span,
        impl_span: Span,
        param_index: ParamIndex,
    },
    PolyImplDifferentNumberOfParams {
        poly_params: usize,
        poly_span: Span,
        impl_params: usize,
        impl_span: Span,
    },

    // If it's `#[poly] fn foo<T>(_: Int) -> T;` and
    // `#[impl] fn foo_impl<T>(_: String) -> T;`, the error would be
    // `CannotImplPoly { poly_type: Int, poly_span: foo_span, impl_type: String, impl_span: foo_impl_span, param_index: ParamIndex::Param(0) }`
    CannotImplPoly {
        poly_type: Type,
        poly_span: Span,
        impl_type: Type,
        impl_span: Span,
        param_index: ParamIndex,
    },

    MultiplePolyCandidates {
        call: Span,
        poly_def: Span,
        candidates: Vec<Span>,
    },
    MissingStructFields {
        span: Span,
        struct_name: InternedString,

        // If it's an enum-variant, the enum name is stored here.
        enum_name: Option<InternedString>,

        missing_fields: Vec<InternedString>,
    },
    ImpureCallInPureContext {
        call_spans: HashMap<FuncEffect, Vec<Span>>,
        keyword_span: Span,
        context: ExprContext,
        context_effect: FuncEffect,
    },

    // warning by default
    NoImpureCallInImpureContext {
        effect_keyword_span: Span,
        context_effect: FuncEffect,
    },

    // This is an ICE.
    TryToSolveGenericParam {
        expected: Type,
        expected_span: Option<Span>,
        got: Type,
        got_span: Option<Span>,
        context: ErrorContext,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ExprContext {
    TopLevelLet,
    InlineLet,
    FuncDefaultValue,
    StructDefaultValue,
    TopLevelFunc,
    InlineFunc,
    Lambda,
    TopLevelAssert,
    Monomorphization,
}

impl From<LetOrigin> for ExprContext {
    fn from(o: LetOrigin) -> ExprContext {
        match o {
            LetOrigin::TopLevel => ExprContext::TopLevelLet,
            LetOrigin::Inline => ExprContext::InlineLet,
            LetOrigin::FuncDefaultValue => ExprContext::FuncDefaultValue,
            LetOrigin::StructDefaultValue => ExprContext::StructDefaultValue,
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
            FuncOrigin::AssociatedFunc => ExprContext::TopLevelFunc,
            FuncOrigin::Monomorphization => ExprContext::Monomorphization,
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
    InferTypeAnnot,
    VerifyTypeAnnot,
    ListElementEqual,
    FuncArgs,
    StructFields,
    EqValueEqual,
    NeqValueEqual,
    OrPatternEqual,
    OrPatternNameBinding(InternedString),
    RangePatternEqual,
    TypeAssertion,
    FieldUpdate,
    EqualGenericParams {
        def: Span,
        call: Span,
        i: usize,
        j: usize,
    },

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
            ErrorContext::InferTypeAnnot => Some(String::from("There's an error while doing type-inference.")),
            ErrorContext::VerifyTypeAnnot => Some(String::from("A value's type annotation and its actual type do not match.")),
            ErrorContext::ListElementEqual => Some(String::from("All elements of a list must have the same type.")),
            ErrorContext::FuncArgs => Some(String::from("Arguments of this function are incorrect.")),
            ErrorContext::StructFields => Some(String::from("Fields of this struct are incorrect.")),
            ErrorContext::EqValueEqual => Some(String::from("Lhs and rhs of `==` operator must have the same type.")),
            ErrorContext::NeqValueEqual => Some(String::from("Lhs and rhs of `!=` operator must have the same type.")),
            ErrorContext::OrPatternEqual => Some(String::from("Lhs and rhs of `|` pattern must have the same type.")),
            ErrorContext::OrPatternNameBinding(name) => Some(format!(
                "Name `{}` is bound multiple times in `|` pattern, but they have different types.",
                name.unintern_or_default(intermediate_dir),
            )),
            ErrorContext::RangePatternEqual => Some(String::from("Lhs and rhs of `..` pattern must have the same type.")),
            ErrorContext::TypeAssertion => Some(String::from("Asserted type and the actual type are different.")),
            ErrorContext::FieldUpdate => Some(String::from("In a field-update expression, the type of the value and the field have to be the same.")),
            ErrorContext::EqualGenericParams { .. } => None,
            ErrorContext::InferedAgain { .. } => Some(String::from("I infered a type of the same value multiple times, and got different results.")),
            ErrorContext::Deep => Some(String::from("A contradiction is found while solving a chain of type-equations. There must be type error somewhere, but I can't find the exact location.")),
            ErrorContext::None => None,
        }
    }
}

impl Session {
    pub fn type_error_to_general_error(&self, error: TypeError) -> Error {
        match error {
            TypeError::UnexpectedType {
                expected,
                expected_span,
                got,
                got_span,
                context,
            } => {
                let mut spans = vec![];
                let expected_type = self.render_type(&expected);
                let got_type = self.render_type(&got);

                if let ErrorContext::InferedAgain { type_var } = &context {
                    match type_var {
                        Type::Var { def_span, is_return } => {
                            spans.push(RenderableSpan {
                                span: def_span.clone(),
                                auxiliary: false,
                                note: Some(format!(
                                    "You didn't annotate the {}, so I tried to infer it. Some information says the type is `{}`, while another information says it's `{}`. Perhaps add a type annotation?",
                                    if *is_return { "return type of this function" } else { "type of this value" },
                                    expected_type,
                                    got_type,
                                )),
                            });
                        },
                        Type::GenericArg { call, generic } => {
                            spans.push(RenderableSpan {
                                span: call.clone(),
                                auxiliary: false,
                                note: Some(format!(
                                    "This is a generic function, so I tried to infer its type arguments, but there's a problem with `{}`. Some information says `{}`'s type is `{}`, while other says it's `{}`.",
                                    self.span_to_string(generic).unwrap_or_else(|| String::from("???")),
                                    self.span_to_string(generic).unwrap_or_else(|| String::from("???")),
                                    expected_type,
                                    got_type,
                                )),
                            });
                            spans.push(RenderableSpan {
                                span: generic.clone(),
                                auxiliary: true,
                                note: Some(format!(
                                    "Type parameter `{}` is defined here.",
                                    self.span_to_string(generic).unwrap_or_else(|| String::from("???")),
                                )),
                            });
                        },
                        _ => unreachable!(),
                    }

                    if let Some(span) = expected_span {
                        spans.push(RenderableSpan {
                            span: span.clone(),
                            auxiliary: false,
                            note: Some(format!("This information says the type is `{expected_type}`.")),
                        });
                    }

                    if let Some(span) = got_span {
                        spans.push(RenderableSpan {
                            span: span.clone(),
                            auxiliary: false,
                            note: Some(format!("This information says the type is `{got_type}`.")),
                        });
                    }
                }

                else {
                    if let Some(span) = expected_span {
                        let note = if let ErrorContext::FieldUpdate = context {
                            format!("The type of this field is `{expected_type}`.")
                        } else if let ErrorContext::MatchScrutinee = context {
                            format!("The scrutinee of the match expression has type `{expected_type}`.")
                        } else if let ErrorContext::EqualGenericParams { .. } = context {
                            format!("This value has type `{expected_type}`.")
                        } else {
                            format!(
                                "The value should have type `{expected_type}`{}.",
                                if let ErrorContext::VerifyTypeAnnot = context {
                                    ", according to this type annotation"
                                } else {
                                    ""
                                },
                            )
                        };

                        spans.push(RenderableSpan {
                            span: span.clone(),
                            auxiliary: true,
                            note: Some(note),
                        });
                    }

                    if let Some(span) = got_span {
                        spans.push(RenderableSpan {
                            span: span.clone(),
                            auxiliary: false,
                            note: Some(format!("This value is expected to have type `{expected_type}`, but has type `{got_type}`.")),
                        });
                    }
                }

                if let ErrorContext::EqualGenericParams { def, call, i, j } = &context {
                    spans.push(RenderableSpan {
                        span: call.clone(),
                        auxiliary: true,
                        note: Some(format!(
                            "The {} and {} arguments of this function should have the same type.",
                            to_ordinal(*i + 1),
                            to_ordinal(*j + 1),
                        )),
                    });
                    spans.push(RenderableSpan {
                        span: def.clone(),
                        auxiliary: true,
                        note: Some(String::from("The function is defined here.")),
                    });
                }

                Error {
                    kind: ErrorKind::UnexpectedType {
                        expected: expected_type,
                        got: got_type,
                    },
                    spans,
                    note: context.note(&self.intermediate_dir).map(|s| s.to_string()),
                }
            },
            TypeError::WrongNumberOfArgs {
                expected,
                got,
                given_keyword_args,
                call,
                def,
                arg_spans,
            } => {
                // TODO: We can have much better error messages...
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

                let mut spans = vec![
                    RenderableSpan {
                        span: call.clone(),
                        auxiliary: false,
                        note: Some(format!(
                            "It has {} argument{}.",
                            got.len(),
                            if got.len() == 1 { "" } else { "s" },
                        )),
                    }
                ];

                if let Some(def) = def {
                    spans.push(RenderableSpan {
                        span: def.clone(),
                        auxiliary: true,
                        note: Some(format!(
                            "It has {} parameter{}.",
                            expected.len(),
                            if expected.len() == 1 { "" } else { "s" },
                        )),
                    });
                }

                Error {
                    kind: ErrorKind::WrongNumberOfArgs {
                        expected: expected.len(),
                        got: got.len(),
                    },
                    spans,
                    note: None,
                }
            },
            TypeError::WrongNumberOfGenericArgs {
                expected,
                got,
                param_group_span,
                arg_group_span,
            } => Error {
                kind: ErrorKind::WrongNumberOfGenericArgs { expected, got },
                spans: vec![
                    RenderableSpan {
                        span: param_group_span.clone(),
                        auxiliary: true,
                        note: Some(format!(
                            "It has {expected} generic parameter{}.",
                            if expected == 1 { "" } else { "s" },
                        )),
                    },
                    RenderableSpan {
                        span: arg_group_span.clone(),
                        auxiliary: false,
                        note: Some(format!(
                            "You provided {got} generic argument{}.",
                            if got == 1 { "" } else { "s" },
                        )),
                    },
                ],
                note: None,
            },
            TypeError::UnnecessaryGenericArgs {
                def_span,
                num_generic_args,
                r#type,
                call_span,
            } => {
                let mut spans = vec![
                    RenderableSpan {
                        span: def_span,
                        auxiliary: true,
                        note: Some(String::from("This is the definition. There are no generic parameters, right?")),
                    },
                ];

                if let Some(call_span) = call_span {
                    spans.push(RenderableSpan {
                        span: call_span,
                        auxiliary: false,
                        note: Some(format!(
                            "This has type `{}`, which has {num_generic_args} generic argument{}.",
                            self.render_type(&r#type),
                            if num_generic_args == 1 { "" } else { "s" },
                        )),
                    });
                }

                Error { kind: ErrorKind::UnnecessaryGenericArgs, spans, note: None }
            },
            TypeError::MissingGenericArgs {
                def_span,
                num_generic_params,
                r#type,
                call_span,
            } => {
                let mut spans = vec![
                    RenderableSpan {
                        span: def_span,
                        auxiliary: true,
                        note: Some(format!(
                            "This is the definition. Do see {num_generic_params} generic parameter{} here?",
                            if num_generic_params == 1 { "" } else { "s" },
                        )),
                    },
                ];

                if let Some(call_span) = call_span {
                    spans.push(RenderableSpan {
                        span: call_span,
                        auxiliary: false,
                        note: Some(format!(
                            "This has type `{}`, which has no generic arguments.",
                            self.render_type(&r#type),
                        )),
                    });
                }

                Error { kind: ErrorKind::MissingGenericArgs, spans, note: None }
            },
            TypeError::CannotInferType { info, span, is_return } => Error {
                kind: ErrorKind::CannotInferType { info, is_return },
                spans: span.simple_error(),
                note: None,
            },
            TypeError::PartiallyInferedType {
                info,
                span,
                r#type,
                is_return,
            } => Error {
                kind: ErrorKind::PartiallyInferedType { info, r#type: self.render_type(&r#type), is_return },
                spans: span.simple_error(),
                note: None,
            },
            TypeError::CannotInferGenericType { ref call, ref generic, ref func_def } |
            TypeError::PartiallyInferedGenericType { ref call, ref generic, ref func_def, .. } => {
                let generic_id = self.span_to_string(generic);
                let spans = match (func_def.as_ref().map(|def_span| self.func_shapes.get(def_span)), &generic_id) {
                    (Some(Some(func_shape)), Some(generic_id)) => vec![
                        RenderableSpan {
                            span: call.clone(),
                            auxiliary: false,
                            note: Some(format!(
                                "This function has {} type parameter{} ({}), and I cannot infer the type of `{generic_id}`.",
                                func_shape.generics.len(),
                                if func_shape.generics.len() == 1 { "" } else { "s" },
                                comma_list_strs(
                                    &func_shape.generics.iter().map(
                                        |generic_param| generic_param.name.unintern_or_default(&self.intermediate_dir)
                                    ).collect::<Vec<_>>(),
                                    "`",
                                    "`",
                                    "and",
                                ),
                            )),
                        },
                        RenderableSpan {
                            span: generic.clone(),
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
                            r#type: self.render_type(&r#type),
                        },
                        spans,
                        note: None,
                    },
                    _ => unreachable!(),
                }
            },
            TypeError::UnknownField { r#type, field } => match field {
                Field::Name { name, name_span, .. } => Error {
                    kind: ErrorKind::UnknownField {
                        r#type: self.render_type(&r#type),
                        field: name,
                    },
                    spans: name_span.simple_error(),
                    note: None,
                },
                _ => todo!(),
            },
            TypeError::CannotUpdateAssociatedFunc { r#type, name, name_span } => Error {
                kind: ErrorKind::CannotUpdateAssociatedFunc { r#type: self.render_type(&r#type), name },
                spans: vec![RenderableSpan {
                    span: name_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("This is an associated function, not a field.")),
                }],
                note: None,
            },
            TypeError::NotCallable { r#type, func_span } => Error {
                kind: ErrorKind::NotCallable {
                    r#type: self.render_type(&r#type),
                },
                spans: vec![RenderableSpan {
                    span: func_span.clone(),
                    auxiliary: false,
                    note: None,
                }],
                note: None,
            },
            // TODO: based on the poly's def_span, I want it to throw
            //       `CannotApplyInfixOp` or so.
            TypeError::CannotSpecializePolyGeneric {
                call,
                poly_def,
                generics,
                num_candidates,
            } => Error {
                kind: ErrorKind::CannotSpecializePolyGeneric { num_candidates },
                spans: vec![
                    vec![
                        RenderableSpan {
                            span: call.clone(),
                            auxiliary: false,
                            note: Some(format!("Cannot specialize `{}` here.", self.span_to_string(&poly_def).unwrap_or_else(|| String::from("????")))),
                        },
                        RenderableSpan {
                            span: poly_def.clone(),
                            auxiliary: true,
                            note: Some(format!("`{}` is defined here.", self.span_to_string(&poly_def).unwrap_or_else(|| String::from("????")))),
                        },
                    ],
                    generics.iter().map(
                        |(span, r#type)| RenderableSpan {
                            span: span.clone(),
                            auxiliary: true,
                            note: Some(format!("Type parameter `{}` is infered to be `{}`.", self.span_to_string(span).unwrap_or_else(|| String::from("????")), self.render_type(r#type))),
                        }
                    ).collect(),
                ].concat(),
                note: None,
            },
            TypeError::UnexpectedEffect {
                expected_type,
                expected_effect,
                expected_span,
                got_type,
                got_effect,
                got_span,
            } => {
                let mut spans = vec![];
                let expected_type = self.render_type(&expected_type);
                let got_type = self.render_type(&got_type);

                if let Some(span) = expected_span {
                    let note = match expected_effect {
                        FuncEffect::Fn => "It expects a pure function.",
                        FuncEffect::Proc => "It expects a deterministic procedure.",
                        FuncEffect::NdetFn => "It expects a non-deterministic function.",
                        FuncEffect::NdetProc => "It expects a non-deterministic procedure.",
                        FuncEffect::Callable => unreachable!(),
                        FuncEffect::Var(_) => todo!(),
                    }.to_string();

                    spans.push(RenderableSpan {
                        span: span.clone(),
                        auxiliary: true,
                        note: Some(note),
                    });
                }

                if let Some(span) = got_span {
                    let note = match got_effect {
                        FuncEffect::Fn => "This is a pure function.",
                        FuncEffect::Proc => "This is a deterministic procedure.",
                        FuncEffect::NdetFn => "This is a non-deterministic function.",
                        FuncEffect::NdetProc => "This is a non-deterministic procedure.",
                        FuncEffect::Callable => "I'm not sure about its effect.",
                        FuncEffect::Var(_) => todo!(),
                    }.to_string();

                    spans.push(RenderableSpan {
                        span: span.clone(),
                        auxiliary: false,
                        note: Some(note),
                    });
                }

                let note = match (expected_effect, got_effect) {
                    (ex, FuncEffect::Callable) => Some(format!(
                        "If you're sure that this is {}, add a type annotation.",
                        match ex {
                            FuncEffect::Fn => "pure",
                            FuncEffect::Proc => "a procedure",
                            FuncEffect::NdetFn => "a non-deterministic function",
                            FuncEffect::NdetProc => "a non-deterministic function",
                            FuncEffect::Callable => unreachable!(),
                            FuncEffect::Var(_) => todo!(),
                        },
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
            TypeError::CannotInferPolyGenericParam { poly_span, param_index } => Error {
                kind: ErrorKind::CannotInferPolyGenericParam { param_index },
                spans: vec![RenderableSpan {
                    span: poly_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("This function needs a type annotation.")),
                }],
                note: None,
            },
            TypeError::CannotInferPolyGenericImpl { poly_span, impl_span, param_index } => Error {
                kind: ErrorKind::CannotInferPolyGenericImpl { param_index },
                spans: vec![
                    RenderableSpan {
                        span: impl_span.clone(),
                        auxiliary: false,
                        note: Some(String::from("This function needs a type annotation.")),
                    },
                    RenderableSpan {
                        span: poly_span.clone(),
                        auxiliary: true,
                        note: Some(String::from("`#[poly]` is defined here.")),
                    },
                ],
                note: None,
            },
            TypeError::PolyImplDifferentNumberOfParams { poly_params, poly_span, impl_params, impl_span } => Error {
                kind: ErrorKind::PolyImplDifferentNumberOfParams { poly_params, impl_params },
                spans: vec![
                    RenderableSpan {
                        span: impl_span.clone(),
                        auxiliary: false,
                        note: Some(format!("It has {impl_params} parameter{}.", if impl_params == 1 { "" } else { "s" })),
                    },
                    RenderableSpan {
                        span: poly_span.clone(),
                        auxiliary: true,
                        note: Some(format!("It has {poly_params} parameter{}.", if poly_params == 1 { "" } else { "s" })),
                    },
                ],
                note: None,
            },
            TypeError::CannotImplPoly {
                poly_type,
                poly_span,
                impl_type,
                impl_span,
                param_index,
            } => Error {
                kind: ErrorKind::CannotImplPoly {
                    poly_type: self.render_type(&poly_type),
                    impl_type: self.render_type(&impl_type),
                    param_index,
                },
                spans: vec![
                    RenderableSpan {
                        span: impl_span.clone(),
                        auxiliary: false,
                        note: None,
                    },
                    RenderableSpan {
                        span: poly_span.clone(),
                        auxiliary: true,
                        note: Some(String::from("`#[poly]` is defined here.")),
                    },
                ],
                note: None,
            },
            TypeError::MultiplePolyCandidates { call, poly_def, candidates } => {
                let mut spans = vec![
                    RenderableSpan {
                        span: call.clone(),
                        auxiliary: false,
                        note: None,
                    },
                    RenderableSpan {
                        span: poly_def.clone(),
                        auxiliary: true,
                        note: Some(String::from("This is the definition of the #[poly] you're trying to impl.")),
                    },
                ];

                for candidate in candidates.iter() {
                    spans.push(RenderableSpan {
                        span: candidate.clone(),
                        auxiliary: true,
                        note: Some(String::from("This is a valid implementation.")),
                    });
                }

                Error {
                    kind: ErrorKind::MultiplePolyCandidates(candidates.len()),
                    spans,
                    note: None,
                }
            },
            TypeError::MissingStructFields { span, struct_name, enum_name, missing_fields } => Error {
                kind: ErrorKind::MissingStructFields {
                    struct_name,
                    enum_name,
                    missing_fields,
                },
                spans: vec![RenderableSpan {
                    span,
                    auxiliary: false,
                    note: None,
                }],
                note: None,
            },
            TypeError::ImpureCallInPureContext { call_spans, keyword_span, context, context_effect } => {
                let mut spans = vec![];
                let (keyword_note, error_note) = match context {
                    ExprContext::TopLevelLet => (
                        Some(String::from("This is a top-level `let` statement, and it has to be pure. If you want to do impure stuffs, define an impure function.")),
                        None,
                    ),
                    ExprContext::InlineLet => unreachable!(),
                    ExprContext::FuncDefaultValue | ExprContext::StructDefaultValue => (
                        None,
                        Some(String::from("You can't call impure functions when initializing a default value.")),
                    ),
                    ExprContext::TopLevelFunc | ExprContext::InlineFunc | ExprContext::Monomorphization => (
                        Some(format!(
                            "You defined {} here.",
                            match context_effect {
                                FuncEffect::Fn => "a pure function",
                                FuncEffect::Proc => "a deterministic procedure",
                                FuncEffect::NdetFn => "a non-deterministic function",
                                FuncEffect::NdetProc => "a non-deterministic procedure",
                                _ => unreachable!(),
                            },
                        )),
                        None,
                    ),
                    ExprContext::Lambda => (
                        Some(String::from("A lambda function is pure by default. If you want the lambda to be effectful, add `ndet` and/or `proc` keyword before the backslash.")),
                        None,
                    ),
                    ExprContext::TopLevelAssert => (
                        Some(String::from("You can't call effectful functions when asserting something.")),
                        None,
                    ),
                };

                spans.push(RenderableSpan {
                    span: keyword_span.clone(),
                    auxiliary: true,
                    note: keyword_note,
                });

                for (effect, call_spans) in call_spans.iter() {
                    let message = match effect {
                        FuncEffect::Fn => unreachable!(),
                        FuncEffect::Proc => "This is a procedure.",
                        FuncEffect::NdetFn => "This is non-deterministic.",
                        FuncEffect::NdetProc => "This is a non-deterministic procedure.",
                        FuncEffect::Callable => "I cannot infer the effect of this function, so it's treated like a non-deterministic procedure.",
                        _ => unreachable!(),
                    };

                    for call_span in call_spans.iter() {
                        spans.push(RenderableSpan {
                            span: call_span.clone(),
                            auxiliary: false,
                            note: Some(message.to_string()),
                        });
                    }
                }

                Error {
                    kind: ErrorKind::ImpureCallInPureContext { context: context_effect },
                    spans,
                    note: error_note,
                }
            },

            TypeWarning::NoImpureCallInImpureContext { effect_keyword_span, context_effect } => Warning {
                kind: WarningKind::NoImpureCallInImpureContext { context: context_effect },
                spans: vec![RenderableSpan {
                    span: effect_keyword_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("This keyword makes this function effectful.")),
                }],
                note: None,
            },

            TypeError::TryToSolveGenericParam {
                expected,
                expected_span,
                got,
                got_span,
                context,
            } => {
                let mut spans = vec![];
                let lhs = self.render_type(&expected);
                let rhs = self.render_type(&got);

                if let Some(span) = expected_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: true,
                        note: Some(String::from("lhs")),
                    });
                }

                if let Some(span) = got_span {
                    spans.push(RenderableSpan {
                        span,
                        auxiliary: true,
                        note: Some(String::from("rhs")),
                    });
                }

                Error {
                    kind: ErrorKind::InternalCompilerError { id: 90962 },
                    spans,
                    note: Some(format!("(This is for debugging the compiler itself, not your program)\nThe compiler tried to solve type with `Type::GenericParam {{ .. }}`. All the `Type::GenericParam {{ .. }}`s must be lowered to `Type::GenericArg {{ .. }}` beforehand.\nlhs: {lhs}\nrhs: {rhs}")),
                }
            },
        }
    }

    pub fn render_type(&self, r#type: &Type) -> String {
        render_type(
            r#type,
            false,  // verbose
            &self.lang_items,
            &self.intermediate_dir,
            &self.span_string_map,
        ).unwrap_or(String::from("????"))
    }

    pub fn span_to_string(&self, span: &Span) -> Option<String> {
        span_to_string(
            span,
            &self.intermediate_dir,
            &self.span_string_map,
        )
    }
}
