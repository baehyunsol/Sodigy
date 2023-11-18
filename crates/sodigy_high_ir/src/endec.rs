use crate::{Expr, Type};
use sodigy_endec::{Endec, EndecErr};

impl Endec for Type {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.0.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(Type(Expr::decode(buf, ind)?))
    }
}
