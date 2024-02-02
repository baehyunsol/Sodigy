use super::MirSession;
use crate::def::Def;
use crate::error::MirError;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_uid::Uid;
use std::collections::HashMap;

impl Endec for MirSession {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buf, session);
        self.func_defs.encode(buf, session);

        todo!()
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MirSession {
            errors: Vec::<MirError>::decode(buf, index, session)?,
            func_defs: HashMap::<Uid, Def>::decode(buf, index, session)?,
            ..todo!()
        })
    }
}
