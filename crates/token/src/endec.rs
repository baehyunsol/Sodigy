use crate::Formatter;
use sodigy_endec::{DecodeError, Endec};

mod constant;
mod delim;
mod keyword;
mod op;
mod punct;

impl Endec for Formatter {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Formatter::Debug => {
                buffer.push(0);
            },
            Formatter::LowerHex => {
                buffer.push(1);
            },
            Formatter::UpperHex => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((Formatter::Debug, cursor + 1)),
            Some(1) => Ok((Formatter::LowerHex, cursor + 1)),
            Some(2) => Ok((Formatter::UpperHex, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
