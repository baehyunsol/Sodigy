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
            Punct::Factorial => {
                buffer.push(13);
            },
            Punct::At => {
                buffer.push(14);
            },
            Punct::Dollar => {
                buffer.push(15);
            },
            Punct::Backslash => {
                buffer.push(16);
            },
            Punct::And => {
                buffer.push(17);
            },
            Punct::Or => {
                buffer.push(18);
            },
            Punct::Xor => {
                buffer.push(19);
            },
            Punct::AndAnd => {
                buffer.push(20);
            },
            Punct::OrOr => {
                buffer.push(21);
            },
            Punct::Shl => {
                buffer.push(22);
            },
            Punct::Shr => {
                buffer.push(23);
            },
            Punct::Eq => {
                buffer.push(24);
            },
            Punct::Leq => {
                buffer.push(25);
            },
            Punct::Neq => {
                buffer.push(26);
            },
            Punct::Geq => {
                buffer.push(27);
            },
            Punct::Concat => {
                buffer.push(28);
            },
            Punct::Append => {
                buffer.push(29);
            },
            Punct::Prepend => {
                buffer.push(30);
            },
            Punct::DotDot => {
                buffer.push(31);
            },
            Punct::DotDotEq => {
                buffer.push(32);
            },
            Punct::Arrow => {
                buffer.push(33);
            },
            Punct::ReturnType => {
                buffer.push(34);
            },
            Punct::Pipeline => {
                buffer.push(35);
            },
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
            Some(13) => Ok((Punct::Factorial, cursor + 1)),
            Some(14) => Ok((Punct::At, cursor + 1)),
            Some(15) => Ok((Punct::Dollar, cursor + 1)),
            Some(16) => Ok((Punct::Backslash, cursor + 1)),
            Some(17) => Ok((Punct::And, cursor + 1)),
            Some(18) => Ok((Punct::Or, cursor + 1)),
            Some(19) => Ok((Punct::Xor, cursor + 1)),
            Some(20) => Ok((Punct::AndAnd, cursor + 1)),
            Some(21) => Ok((Punct::OrOr, cursor + 1)),
            Some(22) => Ok((Punct::Shl, cursor + 1)),
            Some(23) => Ok((Punct::Shr, cursor + 1)),
            Some(24) => Ok((Punct::Eq, cursor + 1)),
            Some(25) => Ok((Punct::Leq, cursor + 1)),
            Some(26) => Ok((Punct::Neq, cursor + 1)),
            Some(27) => Ok((Punct::Geq, cursor + 1)),
            Some(28) => Ok((Punct::Concat, cursor + 1)),
            Some(29) => Ok((Punct::Append, cursor + 1)),
            Some(30) => Ok((Punct::Prepend, cursor + 1)),
            Some(31) => Ok((Punct::DotDot, cursor + 1)),
            Some(32) => Ok((Punct::DotDotEq, cursor + 1)),
            Some(33) => Ok((Punct::Arrow, cursor + 1)),
            Some(34) => Ok((Punct::ReturnType, cursor + 1)),
            Some(35) => Ok((Punct::Pipeline, cursor + 1)),
            Some(n @ 36..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
