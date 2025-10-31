use crate::{InfixOp, PostfixOp, PrefixOp};
use sodigy_endec::{DecodeError, Endec};

impl Endec for PrefixOp {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            PrefixOp::Not => {
                buffer.push(0);
            },
            PrefixOp::Neg => {
                buffer.push(1);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((PrefixOp::Not, cursor + 1)),
            Some(1) => Ok((PrefixOp::Neg, cursor + 1)),
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for InfixOp {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            InfixOp::Add => {
                buffer.push(0);
            },
            InfixOp::Sub => {
                buffer.push(1);
            },
            InfixOp::Mul => {
                buffer.push(2);
            },
            InfixOp::Div => {
                buffer.push(3);
            },
            InfixOp::Rem => {
                buffer.push(4);
            },
            InfixOp::Shl => {
                buffer.push(5);
            },
            InfixOp::Shr => {
                buffer.push(6);
            },
            InfixOp::Lt => {
                buffer.push(7);
            },
            InfixOp::Eq => {
                buffer.push(8);
            },
            InfixOp::Gt => {
                buffer.push(9);
            },
            InfixOp::Leq => {
                buffer.push(10);
            },
            InfixOp::Neq => {
                buffer.push(11);
            },
            InfixOp::Geq => {
                buffer.push(12);
            },
            InfixOp::Index => {
                buffer.push(13);
            },
            InfixOp::Concat => {
                buffer.push(14);
            },
            InfixOp::Range { inclusive: true } => {
                buffer.push(15);
            },
            InfixOp::Range { inclusive: false } => {
                buffer.push(16);
            },
            InfixOp::BitAnd => {
                buffer.push(17);
            },
            InfixOp::BitOr => {
                buffer.push(18);
            },
            InfixOp::LogicAnd => {
                buffer.push(19);
            },
            InfixOp::LogicOr => {
                buffer.push(20);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(_) => todo!(),
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for PostfixOp {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            PostfixOp::Range { inclusive: true } => {
                buffer.push(0);
            },
            PostfixOp::Range { inclusive: false } => {
                buffer.push(1);
            },
            PostfixOp::QuestionMark => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((PostfixOp::Range { inclusive: true }, cursor + 1)),
            Some(1) => Ok((PostfixOp::Range { inclusive: false }, cursor + 1)),
            Some(2) => Ok((PostfixOp::QuestionMark, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
