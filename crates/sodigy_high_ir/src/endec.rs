use crate::{Expr, Type};
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for Type {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Type(Expr::decode(buf, ind, session)?))
    }
}
