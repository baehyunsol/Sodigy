use crate::session::InternedString;
use crate::span::Span;

pub struct ModDef {
    pub(crate) name: InternedString,

    // it points to `m` of `module`
    pub(crate) span: Span,
}

impl ModDef {
    pub fn new(name: InternedString, span: Span) -> Self {
        ModDef { name, span }
    }
}
