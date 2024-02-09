use crate::IdentWithSpan;
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
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
        self.1.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(IdentWithSpan(
            InternedString::decode(buf, index, session)?,
            SpanRange::decode(buf, index, session)?,
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
