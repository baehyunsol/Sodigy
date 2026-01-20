use crate::Type;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct AssociatedItem {
    pub is_func: bool,
    pub name: InternedString,
    pub name_span: Span,
    pub params: Option<usize>,  // only for associated functions
    pub type_span: Span,
    pub r#type: Type,
}
