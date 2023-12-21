use crate::Delim;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for Delim {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            Delim::Brace => { buf.push(0); },
            Delim::Bracket => { buf.push(1); },
            Delim::Paren => { buf.push(2); },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(Delim::Brace),
                    1 => Ok(Delim::Bracket),
                    2 => Ok(Delim::Paren),
                    3.. => Err(EndecError::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecError::Eof),
        }
    }
}
