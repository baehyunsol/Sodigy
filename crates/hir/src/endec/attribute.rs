use crate::Public;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Public {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (public, cursor) = bool::decode_impl(buffer, cursor)?;
        Ok((Public(public), cursor))
    }
}
