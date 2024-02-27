use super::FieldKind;
use crate::IdentWithSpan;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};

impl Endec for FieldKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            FieldKind::Named(n) => {
                buffer.push(0);
                n.encode(buffer, session);
            },
            FieldKind::Index(n) => {
                buffer.push(1);
                n.encode(buffer, session);
            },
            FieldKind::Range(f, t) => {
                buffer.push(2);
                f.encode(buffer, session);
                t.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(FieldKind::Named(IdentWithSpan::decode(buffer, index, session)?)),
                    1 => Ok(FieldKind::Index(i64::decode(buffer, index, session)?)),
                    2 => Ok(FieldKind::Range(
                        i64::decode(buffer, index, session)?,
                        i64::decode(buffer, index, session)?,
                    )),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl DumpJson for FieldKind {
    fn dump_json(&self) -> JsonObj {
        match self {
            FieldKind::Named(n) => json_key_value_table(vec![
                ("field_name", n.id().dump_json()),
            ]),
            FieldKind::Index(n) => json_key_value_table(vec![
                ("field_index", n.dump_json()),
            ]),
            FieldKind::Range(f, t) => json_key_value_table(vec![
                ("field_range_from", f.dump_json()),
                ("field_range_to", t.dump_json()),
            ]),
        }
    }
}
