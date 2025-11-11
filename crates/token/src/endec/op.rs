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
            InfixOp::Append => {
                buffer.push(15);
            },
            InfixOp::Prepend => {
                buffer.push(16);
            },
            InfixOp::Range { inclusive: true } => {
                buffer.push(17);
            },
            InfixOp::Range { inclusive: false } => {
                buffer.push(18);
            },
            InfixOp::BitAnd => {
                buffer.push(19);
            },
            InfixOp::BitOr => {
                buffer.push(20);
            },
            InfixOp::LogicAnd => {
                buffer.push(21);
            },
            InfixOp::LogicOr => {
                buffer.push(22);
            },
            InfixOp::Xor => {
                buffer.push(23);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((InfixOp::Add, cursor + 1)),
            Some(1) => Ok((InfixOp::Sub, cursor + 1)),
            Some(2) => Ok((InfixOp::Mul, cursor + 1)),
            Some(3) => Ok((InfixOp::Div, cursor + 1)),
            Some(4) => Ok((InfixOp::Rem, cursor + 1)),
            Some(5) => Ok((InfixOp::Shl, cursor + 1)),
            Some(6) => Ok((InfixOp::Shr, cursor + 1)),
            Some(7) => Ok((InfixOp::Lt, cursor + 1)),
            Some(8) => Ok((InfixOp::Eq, cursor + 1)),
            Some(9) => Ok((InfixOp::Gt, cursor + 1)),
            Some(10) => Ok((InfixOp::Leq, cursor + 1)),
            Some(11) => Ok((InfixOp::Neq, cursor + 1)),
            Some(12) => Ok((InfixOp::Geq, cursor + 1)),
            Some(13) => Ok((InfixOp::Index, cursor + 1)),
            Some(14) => Ok((InfixOp::Concat, cursor + 1)),
            Some(15) => Ok((InfixOp::Append, cursor + 1)),
            Some(16) => Ok((InfixOp::Prepend, cursor + 1)),
            Some(17) => Ok((InfixOp::Range { inclusive: true }, cursor + 1)),
            Some(18) => Ok((InfixOp::Range { inclusive: false }, cursor + 1)),
            Some(19) => Ok((InfixOp::BitAnd, cursor + 1)),
            Some(20) => Ok((InfixOp::BitOr, cursor + 1)),
            Some(21) => Ok((InfixOp::LogicAnd, cursor + 1)),
            Some(22) => Ok((InfixOp::LogicOr, cursor + 1)),
            Some(23) => Ok((InfixOp::Xor, cursor + 1)),
            Some(n @ 24..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
