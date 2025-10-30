use crate::InternedString;
use sodigy_endec::{DecodeError, Endec};

impl Endec for InternedString {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = u128::decode_impl(buffer, cursor)?;
        Ok((InternedString(id), cursor))
    }
}
