use super::UniversalError;
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_span::SpanRange;

impl Endec for UniversalError {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.context.encode(buf, session);
        self.message.encode(buf, session);
        self.is_warning.encode(buf, session);
        self.spans.encode(buf, session);
        self.show_span.encode(buf, session);
        self.hash.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(UniversalError {
            context: String::decode(buf, index, session)?,
            message: String::decode(buf, index, session)?,
            is_warning: bool::decode(buf, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buf, index, session)?,
            show_span: bool::decode(buf, index, session)?,
            hash: u64::decode(buf, index, session)?,
        })
    }
}
