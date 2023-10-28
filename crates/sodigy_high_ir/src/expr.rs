use crate::names::NameOrigin;
use sodigy_span::SpanRange;
use sodigy_intern::InternedString;

mod lower;

pub struct Expr {
    kind: ExprKind,
    span: SpanRange,
}

pub enum ExprKind {
    Identifier(InternedString, NameOrigin),
}
