use crate::Punct;
use sodigy_endec::{DecodeError, Endec};

impl Endec for Punct {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Punct::Add => {
                buffer.push(0);
            },
            Punct::Sub => {
                buffer.push(1);
            },
            Punct::Mul => {
                buffer.push(2);
            },
            Punct::Div => {
                buffer.push(3);
            },
            Punct::Rem => {
                buffer.push(4);
            },
            Punct::Colon => {
                buffer.push(5);
            },
            Punct::Semicolon => {
                buffer.push(6);
            },
            Punct::Assign => {
                buffer.push(7);
            },
            Punct::Lt => {
                buffer.push(8);
            },
            Punct::Gt => {
                buffer.push(9);
            },
            Punct::Comma => {
                buffer.push(10);
            },
            Punct::Dot => {
                buffer.push(11);
            },
            Punct::QuestionMark => {
                buffer.push(12);
            },
            _ => panic!("TODO: {self:?}"),
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((Punct::Add, cursor + 1)),
            Some(1) => Ok((Punct::Sub, cursor + 1)),
            Some(2) => Ok((Punct::Mul, cursor + 1)),
            Some(3) => Ok((Punct::Div, cursor + 1)),
            Some(4) => Ok((Punct::Rem, cursor + 1)),
            Some(5) => Ok((Punct::Colon, cursor + 1)),
            Some(6) => Ok((Punct::Semicolon, cursor + 1)),
            Some(7) => Ok((Punct::Assign, cursor + 1)),
            Some(8) => Ok((Punct::Lt, cursor + 1)),
            Some(9) => Ok((Punct::Gt, cursor + 1)),
            Some(10) => Ok((Punct::Comma, cursor + 1)),
            Some(11) => Ok((Punct::Dot, cursor + 1)),
            Some(12) => Ok((Punct::QuestionMark, cursor + 1)),
            Some(n @ 13..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
