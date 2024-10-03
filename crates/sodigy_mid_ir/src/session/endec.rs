use super::MirSession;
use crate::error::MirError;
use crate::func::Func;
use crate::warn::MirWarning;
use sodigy_config::CompilerOption;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_error::UniversalError;
use sodigy_high_ir::StructInfo;
use sodigy_intern::{InternSession, InternedString};
use sodigy_session::SessionSnapshot;
use sodigy_uid::Uid;
use std::collections::HashMap;

impl Endec for MirSession {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buffer, session);
        self.warnings.encode(buffer, session);
        self.func_defs.encode(buffer, session);
        self.struct_defs.encode(buffer, session);
        self.uid_name_map.encode(buffer, session);
        self.snapshots.encode(buffer, session);
        self.compiler_option.encode(buffer, session);
        self.previous_errors.encode(buffer, session);
        self.previous_warnings.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MirSession {
            errors: Vec::<MirError>::decode(buffer, index, session)?,
            warnings: Vec::<MirWarning>::decode(buffer, index, session)?,
            interner: InternSession::new(),
            func_defs: HashMap::<Uid, Func>::decode(buffer, index, session)?,
            struct_defs: HashMap::<Uid, StructInfo>::decode(buffer, index, session)?,
            curr_lowering_func: None,
            local_value_table: HashMap::new(),
            uid_name_map: HashMap::<Uid, InternedString>::decode(buffer, index, session)?,
            snapshots: Vec::<SessionSnapshot>::decode(buffer, index, session)?,
            compiler_option: CompilerOption::decode(buffer, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buffer, index, session)?,
            previous_warnings: Vec::<UniversalError>::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for MirSession {
    fn dump_json(&self) -> JsonObj {
        todo!()
    }
}
