use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

pub struct ModDef {
    pub(crate) name: InternedString,

    // it points to `m` of `module`
    pub(crate) def_span: Span,
    pub(crate) name_span: Span,
}

impl ModDef {
    pub fn new(name: InternedString, def_span: Span, name_span: Span) -> Self {
        ModDef { name, def_span, name_span }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        format!("module `{}`;", self.name.to_string(session))
    }
}
