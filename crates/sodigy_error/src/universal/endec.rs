use super::UniversalError;
use smallvec::SmallVec;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_span::SpanRange;

impl Endec for UniversalError {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.context.encode(buf, session);
        self.message.encode(buf, session);
        self.is_warning.encode(buf, session);
        self.spans.encode(buf, session);
        self.show_span.encode(buf, session);
        self.hash.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(UniversalError {
            context: String::decode(buf, index, session)?,
            message: String::decode(buf, index, session)?,
            is_warning: bool::decode(buf, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buf, index, session)?,
            show_span: bool::decode(buf, index, session)?,
            hash: u64::decode(buf, index, session)?,
        })
    }
}

impl DumpJson for UniversalError {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("context", self.context.dump_json()),
            ("message", self.message.dump_json()),
            ("is_warning", self.is_warning.dump_json()),
            ("spans", self.spans.dump_json()),
        ])
    }
}
