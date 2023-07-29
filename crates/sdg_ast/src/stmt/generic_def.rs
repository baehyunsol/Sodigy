use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

#[cfg(test)]
use crate::utils::assert_identifier;

pub struct GenericDef {
    pub(crate) name: InternedString,
    pub(crate) span: Span,
}

impl GenericDef {
    pub fn new(name: InternedString, span: Span) -> Self {
        GenericDef { name, span }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        #[cfg(test)]
        assert_identifier(self.span.dump(session));

        self.name.to_string(session)
    }
}
