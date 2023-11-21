use super::Punct;
use sodigy_endec::{Endec, EndecErr, EndecSession};

impl Endec for Punct {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        todo!()
    }
}
