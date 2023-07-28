use crate::ast::NameScope;
use crate::session::{InternedString, LocalParseSession};

#[derive(Clone)]
pub struct Pattern {
    //
}

impl Pattern {
    pub fn get_name_bindings(&self) -> Vec<InternedString> {
        todo!()
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        todo!()
    }

    // a `Pattern` may include
    //   - enum name, enum variant name, struct name, const
    // a `Pattern` may not include
    //   - local val, func call, 
    pub fn resolve_names(&mut self, scope: &NameScope, session: &LocalParseSession) {
        todo!()
    }
}
