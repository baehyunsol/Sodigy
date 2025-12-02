use crate::Pattern;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Pattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        todo!()
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}
