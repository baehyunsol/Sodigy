use crate::{Expr, Type};
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for Type {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Type(Expr::decode(buffer, index, session)?))
    }
}
