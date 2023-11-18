use crate::Uid;
use sodigy_endec::{Endec, EndecErr};

impl Endec for Uid {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.0.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(Uid(u128::decode(buf, ind)?))
    }
}