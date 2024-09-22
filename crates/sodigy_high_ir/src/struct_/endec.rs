use super::StructInfo;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_intern::InternedString;
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

impl Endec for StructInfo {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.struct_name.encode(buffer, session);
        self.field_names.encode(buffer, session);
        self.struct_uid.encode(buffer, session);
        self.constructor_uid.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(StructInfo {
            struct_name: IdentWithSpan::decode(buffer, index, session)?,
            field_names: Vec::<InternedString>::decode(buffer, index, session)?,
            struct_uid: Uid::decode(buffer, index, session)?,
            constructor_uid: Uid::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for StructInfo {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("struct_name", self.struct_name.id().to_string().dump_json()),
            ("field_names", self.field_names.iter().map(
                |field_name| field_name.to_string()
            ).collect::<Vec<_>>().dump_json()),
            ("struct_uid", self.struct_uid.dump_json()),
            ("constructor_uid", self.constructor_uid.dump_json()),
        ])
    }
}
