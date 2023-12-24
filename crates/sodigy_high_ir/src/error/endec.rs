use super::HirError;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for HirError {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}
