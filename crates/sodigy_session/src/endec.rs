use crate::{SessionDependency, SessionSnapshot};
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for SessionDependency {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.path.encode(buffer, session);
        self.last_modified_at.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(SessionDependency {
            path: String::decode(buffer, index, session)?,
            last_modified_at: u64::decode(buffer, index, session)?,
        })
    }
}

impl Endec for SessionSnapshot {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buffer, session);
        self.warnings.encode(buffer, session);
        self.results.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(SessionSnapshot {
            errors: usize::decode(buffer, index, session)?,
            warnings: usize::decode(buffer, index, session)?,
            results: usize::decode(buffer, index, session)?,
        })
    }
}
