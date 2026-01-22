use crate::Type;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct AssociatedItem {
    pub kind: AssociatedItemKind,
    pub name: InternedString,
    pub name_span: Span,
    pub is_pure: Option<bool>,  // only for associated functions
    pub params: Option<usize>,  // only for associated functions
    pub type_span: Span,
    pub r#type: Type,
}

#[derive(Clone, Copy, Debug)]
pub enum AssociatedItemKind {
    Func,
    Let,

    // These are not really associated items, but inter-hir treat these
    // like associated items so that it can generate better error messages.
    Field,
    Variant,
}
