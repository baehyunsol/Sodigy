use crate::QuoteKind;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for QuoteKind {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            QuoteKind::Double => { buffer.push(0); },
            QuoteKind::Single => { buffer.push(1); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(QuoteKind::Double),
                    1 => Ok(QuoteKind::Single),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
