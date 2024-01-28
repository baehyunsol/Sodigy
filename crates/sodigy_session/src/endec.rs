use crate::SessionSnapshot;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for SessionSnapshot {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.errors.encode(buf, session);
        self.warnings.encode(buf, session);
        self.results.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(SessionSnapshot {
            errors: usize::decode(buf, index, session)?,
            warnings: usize::decode(buf, index, session)?,
            results: usize::decode(buf, index, session)?,
        })
    }
}
