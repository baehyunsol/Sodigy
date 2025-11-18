use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Poly {
    pub decorator_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub has_default_impl: bool,

    // inter-hir will fill this
    pub impls: Vec<Span>,
}
