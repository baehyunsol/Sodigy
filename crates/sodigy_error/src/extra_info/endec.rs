use super::ExtraErrInfo;
use crate::ErrorContext;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};

impl Endec for ExtraErrInfo {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.msg.encode(buf, session);
        self.context.encode(buf, session);
        self.show_span.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ExtraErrInfo {
            msg: String::decode(buf, index, session)?,
            context: ErrorContext::decode(buf, index, session)?,
            show_span: bool::decode(buf, index, session)?,
        })
    }
}

impl DumpJson for ExtraErrInfo {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("message", self.msg.dump_json()),
            ("context", format!("{:?}", self.context).dump_json()),
        ])
    }
}
