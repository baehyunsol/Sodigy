use crate::Expr;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub value: Expr,
}
