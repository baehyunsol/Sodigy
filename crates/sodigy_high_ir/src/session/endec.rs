use super::HirSession;
use crate::error::HirError;
use crate::func::Func;
use crate::module::Module;
use crate::warn::HirWarning;
use log::info;
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_error::UniversalError;
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
        self.previous_errors.encode(buf, session);

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
            previous_errors: Vec::<UniversalError>::decode(buf, index, session)?,
            ..HirSession::new()
        })
    }
}

impl DumpJson for HirSession {
    fn dump_json(&self) -> JsonObj {
        info!("HirSession::dump_json()");

        let errors = self.errors.dump_json();
        let warnings = self.warnings.dump_json();
        let func_defs = self.func_defs.values().map(|f| f.dump_json()).collect::<Vec<_>>().dump_json();
        let imported_names = self.imported_names.dump_json();
        let modules = self.modules.dump_json();
        let previous_errors = self.previous_errors.dump_json();

        json_key_value_table(vec![
            ("errors", errors),
            ("warnings", warnings),
            ("definitions", func_defs),
            ("imported_names", imported_names),
            ("modules", modules),
            ("errors_from_previous_session", previous_errors),
        ])
    }
}
