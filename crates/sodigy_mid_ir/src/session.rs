use crate::def::Def;
use crate::error::MirError;
use sodigy_uid::Uid;
use std::collections::HashMap;

pub struct MirSession {
    errors: Vec<MirError>,
    func_defs: HashMap<Uid, Def>,
}

impl MirSession {
    pub fn push_error(&mut self, e: MirError) {
        self.errors.push(e);
    }
}
