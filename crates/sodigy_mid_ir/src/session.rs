use crate::error::MirError;

pub struct MirSession {
    errors: Vec<MirError>,
}

impl MirSession {
    pub fn push_error(&mut self, e: MirError) {
        self.errors.push(e);
    }
}
