use crate::Uid;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for Uid {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Uid(u128::decode(buf, index, session)?))
    }
}