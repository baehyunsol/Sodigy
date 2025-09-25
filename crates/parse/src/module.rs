use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Module {
    pub name: InternedString,
    pub name_span: Span,
}
