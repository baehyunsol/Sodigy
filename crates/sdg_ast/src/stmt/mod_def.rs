use crate::session::{InternedString, LocalParseSession};
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

    pub fn dump(&self, session: &LocalParseSession) -> String {
        format!("module `{}`;", self.name.to_string(session))
    }
}
