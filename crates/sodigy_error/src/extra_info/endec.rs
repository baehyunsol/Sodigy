use super::ExtraErrorInfo;
use crate::ErrorContext;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};

impl Endec for ExtraErrorInfo {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.extra_messages.encode(buffer, session);
        self.context.encode(buffer, session);
        self.show_span.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ExtraErrorInfo {
            extra_messages: Vec::<String>::decode(buffer, index, session)?,
            context: ErrorContext::decode(buffer, index, session)?,
            show_span: bool::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for ExtraErrorInfo {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("extra_messages", self.extra_messages.dump_json()),
            ("context", format!("{:?}", self.context).dump_json()),
        ])
    }
}
