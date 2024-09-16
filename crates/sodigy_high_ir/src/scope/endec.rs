use super::ScopedLet;
use crate::Type;
use crate::expr::Expr;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_parse::IdentWithSpan;

impl Endec for ScopedLet {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.value.encode(buffer, session);
        self.ty.encode(buffer, session);
        self.is_real.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ScopedLet {
            name: IdentWithSpan::decode(buffer, index, session)?,
            value: Expr::decode(buffer, index, session)?,
            ty: Option::<Type>::decode(buffer, index, session)?,
            is_real: bool::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for ScopedLet {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("name", self.name.id().dump_json()),
            ("value", self.value.dump_json()),
            ("type_annotation", self.ty.dump_json()),
        ])
    }
}
