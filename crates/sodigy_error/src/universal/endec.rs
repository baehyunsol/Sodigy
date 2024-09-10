use crate::Stage;
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
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.context.encode(buffer, session);
        self.message.encode(buffer, session);
        self.is_warning.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.show_span.encode(buffer, session);
        self.hash.encode(buffer, session);
        self.stage.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(UniversalError {
            context: String::decode(buffer, index, session)?,
            message: String::decode(buffer, index, session)?,
            is_warning: bool::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            show_span: bool::decode(buffer, index, session)?,
            hash: u64::decode(buffer, index, session)?,
            stage: Stage::decode(buffer, index, session)?,
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
