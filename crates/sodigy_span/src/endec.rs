use crate::SpanRange;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for SpanRange {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        // EndecSession will update the FileSession when decoding
        session.register_file_hash(self.file);

        self.file.encode(buf, session);
        self.start.encode(buf, session);
        self.end.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(SpanRange {
            file: u64::decode(buf, index, session)?,
            start: usize::decode(buf, index, session)?,
            end: usize::decode(buf, index, session)?,
        })
    }
}
