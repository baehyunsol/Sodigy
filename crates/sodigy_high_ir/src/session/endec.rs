use super::HirSession;
use crate::error::HirError;
use crate::func::Func;
use crate::module::Module;
use crate::struct_::StructInfo;
use crate::warn::HirWarning;
use log::info;
use sodigy_config::CompilerOption;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_error::UniversalError;
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::IdentWithSpan;
use sodigy_session::SessionSnapshot;
use sodigy_uid::Uid;
use std::collections::HashMap;

impl Endec for HirSession {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        // There's no point in encoding intern_session and field_exprs

        self.errors.encode(buffer, session);
        self.warnings.encode(buffer, session);
        self.tmp_names.encode(buffer, session);
        self.func_defs.encode(buffer, session);
        self.struct_defs.encode(buffer, session);
        self.imported_names.encode(buffer, session);
        self.modules.encode(buffer, session);
        self.snapshots.encode(buffer, session);
        self.compiler_option.encode(buffer, session);
        self.previous_errors.encode(buffer, session);
        self.previous_warnings.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        // There's no point in decoding intern_session and field_exprs

        Ok(HirSession {
            errors: Vec::<HirError>::decode(buffer, index, session)?,
            warnings: Vec::<HirWarning>::decode(buffer, index, session)?,
            interner: InternSession::new(),
            tmp_names: Vec::<(InternedString, bool)>::decode(buffer, index, session)?,
            field_exprs: Vec::new(),
            func_defs: HashMap::<Uid, Func>::decode(buffer, index, session)?,
            struct_defs: HashMap::<Uid, StructInfo>::decode(buffer, index, session)?,
            imported_names: Vec::<IdentWithSpan>::decode(buffer, index, session)?,
            modules: Vec::<Module>::decode(buffer, index, session)?,
            snapshots: Vec::<SessionSnapshot>::decode(buffer, index, session)?,
            compiler_option: CompilerOption::decode(buffer, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buffer, index, session)?,
            previous_warnings: Vec::<UniversalError>::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for HirSession {
    fn dump_json(&self) -> JsonObj {
        info!("HirSession::dump_json()");

        let errors = self.errors.dump_json();
        let warnings = self.warnings.dump_json();

        let mut func_defs = self.func_defs.values().collect::<Vec<_>>();
        func_defs.sort_by_key(|f| f.name.span());
        let func_defs = func_defs.iter().map(|f| f.dump_json()).collect::<Vec<_>>().dump_json();

        let mut struct_defs = self.struct_defs.values().collect::<Vec<_>>();
        struct_defs.sort_by_key(|s| s.struct_name.span());
        let struct_defs = struct_defs.iter().map(|s| s.dump_json()).collect::<Vec<_>>().dump_json();

        let imported_names = self.imported_names.dump_json();
        let modules = self.modules.dump_json();
        let previous_errors = self.previous_errors.dump_json();

        json_key_value_table(vec![
            ("errors", errors),
            ("warnings", warnings),
            ("definitions", func_defs),
            ("structs", struct_defs),
            ("imported_names", imported_names),
            ("modules", modules),
            ("errors_from_previous_session", previous_errors),
            ("rendered", self.dump_hir().dump_json()),
        ])
    }
}
