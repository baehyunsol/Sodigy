use crate::Error;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Error {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        panic!("TODO: {self:?}")
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}
