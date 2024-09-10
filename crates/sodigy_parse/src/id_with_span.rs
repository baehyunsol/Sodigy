use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

mod endec;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IdentWithSpan(InternedString, SpanRange);

impl IdentWithSpan {
    pub fn new(id: InternedString, span: SpanRange) -> Self {
        IdentWithSpan(id, span)
    }

    pub fn id(&self) -> InternedString {
        self.0
    }

    pub fn span(&self) -> &SpanRange {
        &self.1
    }

    pub fn set_id(&mut self, id: InternedString) {
        self.0 = id;
    }

    pub fn set_span(&mut self, span: SpanRange) {
        self.1 = span;
    }
}
