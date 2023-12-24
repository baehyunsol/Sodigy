use super::HirSession;
use crate::error::HirError;
use crate::func::Func;
use crate::warn::HirWarning;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;
use std::collections::HashMap;

impl Endec for HirSession {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buf, session);
        self.warnings.encode(buf, session);
        self.func_defs.encode(buf, session);

        // There's no point in encoding the other fields
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let mut res = Self::new();

        // There's no point in decoding the other fields
        res.errors = Vec::<HirError>::decode(buf, index, session)?;
        res.warnings = Vec::<HirWarning>::decode(buf, index, session)?;
        res.func_defs = HashMap::<InternedString, Func>::decode(buf, index, session)?;

        Ok(res)
    }
}
