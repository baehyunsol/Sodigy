use crate::Delim;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for Delim {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            Delim::Brace => { buffer.push(0); },
            Delim::Bracket => { buffer.push(1); },
            Delim::Paren => { buffer.push(2); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
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
