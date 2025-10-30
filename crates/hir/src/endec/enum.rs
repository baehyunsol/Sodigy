use crate::Enum;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Enum {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        //
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        Ok((Enum, cursor))
    }
}
