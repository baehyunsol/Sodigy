use crate::Delim;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Delim {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Delim::Parenthesis => {
                buffer.push(0);
            },
            Delim::Bracket => {
                buffer.push(1);
            },
            Delim::Brace => {
                buffer.push(2);
            },
            Delim::Lambda => {
                buffer.push(3);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((Delim::Parenthesis, cursor + 1)),
            Some(1) => Ok((Delim::Bracket, cursor + 1)),
            Some(2) => Ok((Delim::Brace, cursor + 1)),
            Some(3) => Ok((Delim::Lambda, cursor + 1)),
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
