use super::IdentWithSpan;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for IdentWithSpan {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buffer, session);
        self.1.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(IdentWithSpan(
            InternedString::decode(buffer, index, session)?,
            SpanRange::decode(buffer, index, session)?,
        ))
    }
}

impl DumpJson for IdentWithSpan {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("id", self.id().to_string().dump_json()),
            ("span", self.span().dump_json()),
        ])
    }
}

