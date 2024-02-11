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
    json_key_value_table,
};
use sodigy_error::UniversalError;
use sodigy_intern::InternSession;
use sodigy_session::{SessionDependency, SessionSnapshot};
use sodigy_uid::Uid;
use std::collections::HashMap;

impl Endec for MirSession {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buffer, session);
        self.warnings.encode(buffer, session);
        self.func_defs.encode(buffer, session);
        self.snapshots.encode(buffer, session);
        self.dependencies.encode(buffer, session);
        self.previous_errors.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MirSession {
            errors: Vec::<MirError>::decode(buffer, index, session)?,
            warnings: Vec::<MirWarning>::decode(buffer, index, session)?,
            func_defs: HashMap::<Uid, Def>::decode(buffer, index, session)?,
            interner: InternSession::new(),
            snapshots: Vec::<SessionSnapshot>::decode(buffer, index, session)?,
            dependencies: Vec::<SessionDependency>::decode(buffer, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for MirSession {
    fn dump_json(&self) -> JsonObj {
        info!("MirSession::dump_json()");

        let errors = self.errors.dump_json();
        let warnings = self.warnings.dump_json();

        // TODO: dump func_defs
        // it must have a consistent order -> multiple compilation on the same file dumps the same json files

        json_key_value_table(vec![
            ("errors", errors),
            ("warnings", warnings),
        ])
    }
}
