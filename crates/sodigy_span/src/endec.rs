use crate::SpanRange;
use sodigy_endec::{Endec, EndecErr, EndecSession};

impl Endec for SpanRange {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.file.encode(buf, session);
        self.start.encode(buf, session);
        self.end.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(SpanRange {
            file: u64::decode(buf, ind, session)?,
            start: usize::decode(buf, ind, session)?,
            end: usize::decode(buf, ind, session)?,
        })
    }
}
