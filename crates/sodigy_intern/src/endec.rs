use crate::{InternedString, InternedNumeric};
use sodigy_endec::{Endec, EndecErr};

impl Endec for InternedString {
    fn encode(&self, buf: &mut Vec<u8>) {
        // TODO: intern_session is validated between compilations
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        // TODO: intern_session is validated between compilations
        todo!()
    }
}

impl Endec for InternedNumeric {
    fn encode(&self, buf: &mut Vec<u8>) {
        // TODO: intern_session is validated between compilations
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        // TODO: intern_session is validated between compilations
        todo!()
    }
}
