use super::Pattern;
use crate::session::InternedString;
use crate::span::Span;

#[derive(Clone)]
pub struct FieldPatternDef {
    pub(crate) field_name: InternedString,
    pub(crate) field_span: Span,
    pub(crate) pattern: Pattern,
}
