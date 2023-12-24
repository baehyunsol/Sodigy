use crate::QuoteKind;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for QuoteKind {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            QuoteKind::Double => { buf.push(0); },
            QuoteKind::Single => { buf.push(1); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
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
