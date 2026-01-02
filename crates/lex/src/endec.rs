use crate::Session;
use sodigy_endec::{DecodeError, DumpSession, Endec};

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        todo!()
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}

impl DumpSession for Session {
    fn dump_session(&self) -> Vec<u8> {
        let s = format!("{:?}", self.tokens);
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();
        c.output().to_vec()
    }
}
