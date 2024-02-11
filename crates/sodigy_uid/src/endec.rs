use crate::Uid;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};

impl Endec for Uid {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Uid(u128::decode(buffer, index, session)?))
    }
}

impl DumpJson for Uid {
    fn dump_json(&self) -> JsonObj {
        format!("{:x}", self.0).dump_json()
    }
}
