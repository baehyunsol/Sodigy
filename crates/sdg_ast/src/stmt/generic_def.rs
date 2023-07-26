use crate::session::InternedString;
use crate::span::Span;

pub struct GenericDef {
    name: InternedString,
    span: Span,
}

impl GenericDef {
    pub fn new(name: InternedString, span: Span) -> Self {
        GenericDef { name, span }
    }
}
