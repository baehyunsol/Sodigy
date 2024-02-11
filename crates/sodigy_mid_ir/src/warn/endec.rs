use super::MirWarning;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_error::SodigyError;

impl Endec for MirWarning {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}

impl DumpJson for MirWarning {
    fn dump_json(&self) -> JsonObj {
        self.dump_json_impl()
    }
}
