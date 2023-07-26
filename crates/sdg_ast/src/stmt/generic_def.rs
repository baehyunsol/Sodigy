use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

pub struct GenericDef {
    pub(crate) name: InternedString,
    pub(crate) span: Span,
}

impl GenericDef {
    pub fn new(name: InternedString, span: Span) -> Self {
        GenericDef { name, span }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        self.name.to_string(session)
    }
}
