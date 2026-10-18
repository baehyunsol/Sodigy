use crate::Session;
use sodigy_endec::{DecodeError, Endec};

// TODO: just remove this impl
impl Endec for Session {
    fn encode_impl(&self, _: &mut Vec<u8>) {
        todo!()
    }

    fn decode_impl(_: &[u8], _: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}

