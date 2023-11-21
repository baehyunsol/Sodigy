use crate::QuoteKind;
use sodigy_endec::{Endec, EndecErr, EndecSession};

impl Endec for QuoteKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            QuoteKind::Double => {
                buf.push(0);
            },
            QuoteKind::Single => {
                buf.push(1);
            },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(QuoteKind::Double),
                    1 => Ok(QuoteKind::Single),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
