use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;

pub struct VariantDef {
    pub(crate) name: InternedString,
    pub(crate) span: Span,
    pub(crate) fields: Option<Vec<Expr>>,
}

impl VariantDef {
    pub fn new(name: InternedString, span: Span, fields: Vec<Expr>) -> Self {
        VariantDef {
            name, span,
            fields: Some(fields),
        }
    }

    pub fn new_no_field(name: InternedString, span: Span) -> Self {
        VariantDef {
            name, span,
            fields: None,
        }
    }
}
