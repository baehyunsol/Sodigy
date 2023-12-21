use crate::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for IdentWithSpan {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
        self.1.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(IdentWithSpan(
            InternedString::decode(buf, ind, session)?,
            SpanRange::decode(buf, ind, session)?,
        ))
    }
}
