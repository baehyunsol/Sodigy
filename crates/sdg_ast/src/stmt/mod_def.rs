use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

#[cfg(test)]
use crate::utils::assert_identifier;

pub struct ModDef {
    pub(crate) name: InternedString,

    /// keyword `module`
    pub(crate) def_span: Span,
    pub(crate) name_span: Span,
}

impl ModDef {
    pub fn new(name: InternedString, def_span: Span, name_span: Span) -> Self {
        ModDef { name, def_span, name_span }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        #[cfg(test)]
        assert_eq!(self.def_span.dump(session), "module");

        #[cfg(test)]
        assert_identifier(self.name_span.dump(session));

        format!("module `{}`;", self.name.to_string(session))
    }
}
