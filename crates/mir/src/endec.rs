use crate::Session;
use sodigy_endec::{DecodeError, DumpIr, Endec};

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        todo!()
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}

impl DumpIr for Session {
    fn dump_ir(&self) -> Vec<u8> {
        todo!()
    }
}
