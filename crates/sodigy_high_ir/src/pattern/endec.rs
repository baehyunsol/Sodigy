use crate::Type;
use super::{Pattern, PatternKind};
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_span::SpanRange;

impl Endec for Pattern {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.span.encode(buf, session);
        self.ty.encode(buf, session);
        self.bind.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Pattern {
            kind: PatternKind::decode(buf, index, session)?,
            span: SpanRange::decode(buf, index, session)?,
            ty: Option::<Type>::decode(buf, index, session)?,
            bind: Option::<IdentWithSpan>::decode(buf, index, session)?,
        })
    }
}

impl Endec for PatternKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}
