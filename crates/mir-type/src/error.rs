use crate::Type;
use sodigy_error::{Error, ErrorKind};
use sodigy_mir::Session as MirSession;
use sodigy_span::Span;
use sodigy_string::{InternedString, unintern_string};
use sodigy_token::InfixOp;

#[derive(Clone, Debug)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Span,
    pub extra_span: Option<Span>,
    pub context: ErrorContext,
}

#[derive(Clone, Debug)]
pub enum TypeErrorKind {
    UnexpectedType {
        expected: Type,
        got: Type,
    },
    // Since it's a very common error, the compiler tries to
    // give an as helpful error message as possible
    WrongNumberOfArguments {
        expected: Vec<Type>,
        got: Vec<Type>,

        // It has type `Vec<(keyword: InternedString, n: usize)>` where
        // `n`th argument of `got` has keyword `keyword`.
        given_keyword_arguments: Vec<(InternedString, usize)>,

        arg_spans: Vec<Span>,
    },
    CannotInferType {
        id: Option<InternedString>,
    },
    PartiallyInferedType {
        id: Option<InternedString>,
        r#type: Type,
    },
    CannotInferGenericType {
        generic_def_span: Span,
    },
    PartiallyInferedGenericType {
        generic_def_span: Span,
        r#type: Type,
    },
    CannotApplyInfixOp {
        op: InfixOp,
        arg_types: Vec<Type>,
    },
    NotCallable {
        r#type: Type,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorContext {
    AssertConditionBool,
    IfConditionBool,
    IfValueEqual,
    InferTypeAnnotation,
    VerifyTypeAnnotation,
    ListElementEqual,
    FuncArgs,
    EqValueEqual,
    Deep,

    // If there's nothing special about the context,
    // or the error kind tells everything about the context.
    None,
}

impl ErrorContext {
    pub fn message(&self) -> Option<&'static str> {
        match self {
            ErrorContext::AssertConditionBool => Some("An assertion must be a boolean."),
            ErrorContext::IfConditionBool => Some("A condition of an `if` expression must be a boolean."),
            ErrorContext::IfValueEqual => Some("All branches of an `if` expression must have the same type."),
            ErrorContext::InferTypeAnnotation => Some("There's an error while doing type-inference."),
            ErrorContext::VerifyTypeAnnotation => Some("A type annotation and its actual type do not match."),
            ErrorContext::ListElementEqual => Some("All elements of a list must have the same type."),
            ErrorContext::FuncArgs => Some("Arguments of this function are incorrect."),
            ErrorContext::EqValueEqual => Some("Lhs and rhs of `==` operator must have the same type."),
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
        let error_kind = match &error.kind {
            TypeErrorKind::UnexpectedType {
                expected,
                got,
            } => ErrorKind::UnexpectedType {
                expected: self.render_type(expected),
                got: self.render_type(got),
            },
            TypeErrorKind::WrongNumberOfArguments {
                expected,
                got,
                given_keyword_arguments,
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
            TypeErrorKind::CannotInferType { id } => ErrorKind::CannotInferType { id: *id },
            TypeErrorKind::PartiallyInferedType {
                id,
                r#type,
            } => ErrorKind::PartiallyInferedType { id: *id, r#type: self.render_type(r#type) },
            TypeErrorKind::CannotInferGenericType { generic_def_span } => ErrorKind::CannotInferGenericType { id: self.span_to_string(*generic_def_span) },
            TypeErrorKind::PartiallyInferedGenericType {
                generic_def_span,
                r#type,
            } => ErrorKind::PartiallyInferedGenericType { id: self.span_to_string(*generic_def_span), r#type: self.render_type(r#type) },
            TypeErrorKind::CannotApplyInfixOp { op, arg_types } => ErrorKind::CannotApplyInfixOp {
                op: *op,
                arg_types: arg_types.iter().map(|t| self.render_type(t)).collect(),
            },
            _ => todo!(),
        };

        Error {
            kind: error_kind,
            span: error.span,
            extra_span: error.extra_span,
            extra_message: error.context.message().map(|s| s.to_string()),
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
        }
    }

    fn span_to_string(&self, span: Span) -> Option<String> {
        match span {
            Span::Prelude(p) => match unintern_string(p, &self.intermediate_dir) {
                Ok(Some(p)) => Some(String::from_utf8_lossy(&p).to_string()),
                _ => None,
            },
            Span::Range { .. } => match self.span_string_map.as_ref().map(|map| map.get(&span)) {
                Some(Some(s)) => Some(String::from_utf8_lossy(s).to_string()),
                _ => None,
            },
            Span::None => None,
            _ => todo!(),
        }
    }
}
