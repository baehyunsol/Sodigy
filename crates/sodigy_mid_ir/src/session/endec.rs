use super::MirSession;
use crate::def::Def;
use crate::error::MirError;
use crate::warn::MirWarning;
use log::info;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_error::UniversalError;
use sodigy_intern::InternSession;
use sodigy_session::{SessionDependency, SessionSnapshot};
use sodigy_uid::Uid;
use std::collections::HashMap;

impl Endec for MirSession {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buf, session);
        self.warnings.encode(buf, session);
        self.func_defs.encode(buf, session);
        self.snapshots.encode(buf, session);
        self.dependencies.encode(buf, session);
        self.previous_errors.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MirSession {
            errors: Vec::<MirError>::decode(buf, index, session)?,
            warnings: Vec::<MirWarning>::decode(buf, index, session)?,
            func_defs: HashMap::<Uid, Def>::decode(buf, index, session)?,
            interner: InternSession::new(),
            snapshots: Vec::<SessionSnapshot>::decode(buf, index, session)?,
            dependencies: Vec::<SessionDependency>::decode(buf, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buf, index, session)?,
        })
    }
}

impl DumpJson for MirSession {
    fn dump_json(&self) -> JsonObj {
        info!("MirSession::dump_json()");
        todo!()
    }
}
