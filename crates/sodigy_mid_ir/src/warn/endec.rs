use super::MirWarning;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};

impl Endec for MirWarning {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}
