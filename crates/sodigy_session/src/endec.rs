use crate::SessionSnapshot;
use sodigy_endec::{Endec, EndecError, EndecSession};

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
