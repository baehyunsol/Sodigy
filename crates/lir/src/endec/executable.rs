use crate::Executable;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Executable {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        todo!()
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}
