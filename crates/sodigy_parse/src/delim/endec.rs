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

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Delim::Brace),
                    1 => Ok(Delim::Bracket),
                    2 => Ok(Delim::Paren),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
