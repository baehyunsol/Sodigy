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

    // type annotation inside `#[associated(...)]`
    pub type_span: Span,
    pub r#type: Type,
}

impl Default for AssociatedItem {
    fn default() -> AssociatedItem {
        AssociatedItem {
            kind: AssociatedItemKind::Let,
            name: InternedString::dummy(),
            name_span: Span::None,
            is_pure: None,
            params: None,
            type_span: Span::None,
            r#type: Type::Never(Span::None),
        }
    }
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

#[derive(Clone, Debug)]
pub struct AssociatedFunc {
    pub name: InternedString,

    // A struct can have multiple assoc-funcs with the same name.
    pub name_spans: Vec<Span>,

    // The multiple assoc-funcs must have the same number of parameters,
    // and must have the same purity.
    pub params: usize,
    pub is_pure: bool,
}
