use crate::Type;
use sodigy_span::Span;
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
    InfixOpNotApplicable {
        op: InfixOp,
        arg_types: Vec<Type>,
    },

    // `fn foo<T>() -> T = 3;` is `GenericIsNotGeneric { got: Int }`
    GenericIsNotGeneric { got: Type },
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorContext {
    // condition of an `if` expression must be `Bool`
    IfConditionBool,

    // true-value and false-value of `if` expression must be the same
    IfValueEqual,

    // there has to be a type annotation, but there isn't
    // so we're infering the type annotation.
    InferTypeAnnotation,

    // there is a type annotation, so we have to check if it's correct
    VerifyTypeAnnotation,

    // If there's nothing special about the context,
    // or the error kind tells everything about the context.
    None,
}
