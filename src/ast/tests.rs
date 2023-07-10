use super::AST;
use crate::session::LocalParseSession;

mod name_resolve;

impl AST {

    pub fn dump_ast_of_def(&self, def: Vec<u8>, session: &LocalParseSession) -> Option<String> {
        let func_name = if let Some(s) = session.try_intern_string(def) {
            s
        } else {
            return None;
        };

        let func = if let Some(f) = self.defs.get(&func_name) {
            f
        } else {
            return None;
        };

        Some(func.ret_val.to_string(session))
    }

}