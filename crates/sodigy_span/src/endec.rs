use crate::SpanRange;
use sodigy_endec::{Endec, EndecErr};

impl Endec for SpanRange {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.file.encode(buf);
        self.start.encode(buf);
        self.end.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(SpanRange {
            file: u64::decode(buf, ind)?,
            start: usize::decode(buf, ind)?,
            end: usize::decode(buf, ind)?,
        })
    }
}
