use crate::IdentWithSpan;
use sodigy_endec::{Endec, EndecErr};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for IdentWithSpan {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.0.encode(buf);
        self.1.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(IdentWithSpan(
            InternedString::decode(buf, ind)?,
            SpanRange::decode(buf, ind)?,
        ))
    }
}
