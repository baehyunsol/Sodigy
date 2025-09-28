use crate::Expr;
use sodigy_span::Span;
use sodigy_string::InternedString;

pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub value: Expr,
}
