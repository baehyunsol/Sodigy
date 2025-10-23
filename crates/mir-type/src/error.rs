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
    CannotInferType {
        id: Option<InternedString>,
    },
    PartiallyInferedType {
        id: Option<InternedString>,
        r#type: Type,
    },
    InfixOpNotApplicable {
        op: InfixOp,
        arg_types: Vec<Type>,
    },

    // `fn foo<T>() -> T = 3;` is `GenericIsNotGeneric { got: Int }`
    GenericIsNotGeneric { got: Type },
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
            ErrorContext::None => None,
        }
    }
}

pub trait RenderTypeError {
    fn type_error_to_general_error(&self, error: &TypeError) -> Error;
    fn render_type(&self, r#type: &Type) -> String;
    fn span_to_string(&self, span: Span) -> String;
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
            TypeErrorKind::CannotInferType { id } => ErrorKind::CannotInferType { id: *id },
            TypeErrorKind::PartiallyInferedType {
                id,
                r#type,
            } => ErrorKind::PartiallyInferedType { id: *id, r#type: self.render_type(r#type) },
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
            Type::Static(span) | Type::GenericDef(span) => self.span_to_string(*span),
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

    fn span_to_string(&self, span: Span) -> String {
        match span {
            Span::Prelude(p) => {
                let p = unintern_string(p, &self.intermediate_dir).unwrap_or(None).unwrap_or(b"????".to_vec());
                String::from_utf8_lossy(&p).to_string()
            },
            _ => todo!(),
        }
    }
}
