use crate::Type;
use super::{Pattern, PatternKind};
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_span::SpanRange;

impl Endec for Pattern {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.span.encode(buf, session);
        self.ty.encode(buf, session);
        self.bind.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(Pattern {
            kind: PatternKind::decode(buf, ind, session)?,
            span: SpanRange::decode(buf, ind, session)?,
            ty: Option::<Type>::decode(buf, ind, session)?,
            bind: Option::<IdentWithSpan>::decode(buf, ind, session)?,
        })
    }
}

impl Endec for PatternKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        todo!()
    }
}
