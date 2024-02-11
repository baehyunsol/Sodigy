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
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buffer, session);
        self.warnings.encode(buffer, session);
        self.func_defs.encode(buffer, session);
        self.imported_names.encode(buffer, session);
        self.modules.encode(buffer, session);
        self.dependencies.encode(buffer, session);
        self.previous_errors.encode(buffer, session);

        // There's no point in encoding the other fields
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        // There's no point in decoding the other fields
        Ok(HirSession {
            errors: Vec::<HirError>::decode(buffer, index, session)?,
            warnings: Vec::<HirWarning>::decode(buffer, index, session)?,
            func_defs: HashMap::<InternedString, Func>::decode(buffer, index, session)?,
            imported_names: Vec::<IdentWithSpan>::decode(buffer, index, session)?,
            modules: Vec::<Module>::decode(buffer, index, session)?,
            dependencies: Vec::<SessionDependency>::decode(buffer, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buffer, index, session)?,
            ..HirSession::new()
        })
    }
}

impl DumpJson for HirSession {
    fn dump_json(&self) -> JsonObj {
        info!("HirSession::dump_json()");

        let errors = self.errors.dump_json();
        let warnings = self.warnings.dump_json();

        // it has to make sure that `dump_json` of the same code returns the same result
        let mut func_defs = self.func_defs.values().collect::<Vec<_>>();
        func_defs.sort_by_key(|f| f.name.span());
        let func_defs = func_defs.iter().map(|f| f.dump_json()).collect::<Vec<_>>().dump_json();

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
