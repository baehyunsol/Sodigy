use crate::Keyword;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Keyword {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Keyword::As => {
                buffer.push(0);
            },
            Keyword::Assert => {
                buffer.push(1);
            },
            Keyword::Else => {
                buffer.push(2);
            },
            Keyword::Enum => {
                buffer.push(3);
            },
            Keyword::Fn => {
                buffer.push(4);
            },
            Keyword::If => {
                buffer.push(5);
            },
            Keyword::Let => {
                buffer.push(6);
            },
            Keyword::Match => {
                buffer.push(7);
            },
            Keyword::Mod => {
                buffer.push(8);
            },
            Keyword::Pub => {
                buffer.push(9);
            },
            Keyword::Struct => {
                buffer.push(10);
            },
            Keyword::Type => {
                buffer.push(11);
            },
            Keyword::Use => {
                buffer.push(12);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((Keyword::As, cursor + 1)),
            Some(1) => Ok((Keyword::Assert, cursor + 1)),
            Some(2) => Ok((Keyword::Else, cursor + 1)),
            Some(3) => Ok((Keyword::Enum, cursor + 1)),
            Some(4) => Ok((Keyword::Fn, cursor + 1)),
            Some(5) => Ok((Keyword::If, cursor + 1)),
            Some(6) => Ok((Keyword::Let, cursor + 1)),
            Some(7) => Ok((Keyword::Match, cursor + 1)),
            Some(8) => Ok((Keyword::Mod, cursor + 1)),
            Some(9) => Ok((Keyword::Pub, cursor + 1)),
            Some(10) => Ok((Keyword::Struct, cursor + 1)),
            Some(11) => Ok((Keyword::Type, cursor + 1)),
            Some(12) => Ok((Keyword::Use, cursor + 1)),
            Some(n @ 13..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
