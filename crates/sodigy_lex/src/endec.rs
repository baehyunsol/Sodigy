use crate::QuoteKind;
use sodigy_endec::{Endec, EndecErr, EndecSession};

impl Endec for QuoteKind {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            QuoteKind::Double => { buf.push(0); },
            QuoteKind::Single => { buf.push(1); },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, _: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(QuoteKind::Double),
                    1 => Ok(QuoteKind::Single),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
