use crate::session::InternedString;
use crate::span::Span;

mod kind;

pub use kind::ASTErrorKind;

pub struct ASTError {
    kind: ASTErrorKind,
    span1: Span,
    span2: Span,  // optional
}

impl ASTError {

    pub(crate) fn def(name: InternedString, first_def: Span, second_def: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::MultipleDef(name),
            span1: first_def,
            span2: second_def,
        }
    }

    pub(crate) fn deco(span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::DecoratorOnUse,
            span1: span,
            span2: Span::dummy(),
        }
    }

}