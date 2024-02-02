use super::HirSession;
use crate::error::HirError;
use crate::func::Func;
use crate::module::Module;
use crate::warn::HirWarning;
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;
use sodigy_session::SessionDependency;
use std::collections::HashMap;

impl Endec for HirSession {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buf, session);
        self.warnings.encode(buf, session);
        self.func_defs.encode(buf, session);
        self.imported_names.encode(buf, session);
        self.modules.encode(buf, session);
        self.dependencies.encode(buf, session);

        // There's no point in encoding the other fields
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        // There's no point in decoding the other fields
        Ok(HirSession {
            errors: Vec::<HirError>::decode(buf, index, session)?,
            warnings: Vec::<HirWarning>::decode(buf, index, session)?,
            func_defs: HashMap::<InternedString, Func>::decode(buf, index, session)?,
            imported_names: Vec::<IdentWithSpan>::decode(buf, index, session)?,
            modules: Vec::<Module>::decode(buf, index, session)?,
            dependencies: Vec::<SessionDependency>::decode(buf, index, session)?,
            ..HirSession::new()
        })
    }
}
