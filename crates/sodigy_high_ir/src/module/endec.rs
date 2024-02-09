use super::Module;
use crate::attr::Attribute;
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_uid::Uid;

impl Endec for Module {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.uid.encode(buf, session);
        self.attributes.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Module {
            name: IdentWithSpan::decode(buf, index, session)?,
            uid: Uid::decode(buf, index, session)?,
            attributes: Vec::<Attribute>::decode(buf, index, session)?,
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
