use super::Module;
use crate::attr::Attribute;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

impl Endec for Module {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.uid.encode(buffer, session);
        self.attributes.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Module {
            name: IdentWithSpan::decode(buffer, index, session)?,
            uid: Uid::decode(buffer, index, session)?,
            attributes: Vec::<Attribute>::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for Module {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("name", self.name.dump_json()),
            ("uid", self.uid.dump_json()),
            ("attributes", self.attributes.dump_json()),
        ])
    }
}
