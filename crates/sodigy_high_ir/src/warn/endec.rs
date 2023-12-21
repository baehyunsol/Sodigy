use super::HirWarning;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for HirWarning {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}
