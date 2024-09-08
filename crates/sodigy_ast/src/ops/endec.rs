use super::{InfixOp, PostfixOp, PrefixOp};
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;

impl Endec for PrefixOp {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            PrefixOp::Not => { buffer.push(0); },
            PrefixOp::Neg => { buffer.push(1); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(PrefixOp::Not),
                    1 => Ok(PrefixOp::Neg),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for PostfixOp {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            PostfixOp::Range => { buffer.push(0); },
            PostfixOp::QuestionMark => { buffer.push(1); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(PostfixOp::Range),
                    1 => Ok(PostfixOp::QuestionMark),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for InfixOp {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            InfixOp::Add => { buffer.push(0); },
            InfixOp::Sub => { buffer.push(1); },
            InfixOp::Mul => { buffer.push(2); },
            InfixOp::Div => { buffer.push(3); },
            InfixOp::Rem => { buffer.push(4); },
            InfixOp::Eq => { buffer.push(5); },
            InfixOp::Gt => { buffer.push(6); },
            InfixOp::Lt => { buffer.push(7); },
            InfixOp::Ne => { buffer.push(8); },
            InfixOp::Ge => { buffer.push(9); },
            InfixOp::Le => { buffer.push(10); },
            InfixOp::BitwiseAnd => { buffer.push(11); },
            InfixOp::BitwiseOr => { buffer.push(12); },
            InfixOp::LogicalAnd => { buffer.push(13); },
            InfixOp::LogicalOr => { buffer.push(14); },
            InfixOp::Xor => { buffer.push(15); },
            InfixOp::ShiftRight => { buffer.push(16); },
            InfixOp::ShiftLeft => { buffer.push(17); },
            InfixOp::Index => { buffer.push(18); },
            InfixOp::Concat => { buffer.push(19); },
            InfixOp::Range => { buffer.push(20); },
            InfixOp::InclusiveRange => { buffer.push(21); },
            InfixOp::FieldModifier(id) => {
                buffer.push(22);
                id.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(InfixOp::Add),
                    1 => Ok(InfixOp::Sub),
                    2 => Ok(InfixOp::Mul),
                    3 => Ok(InfixOp::Div),
                    4 => Ok(InfixOp::Rem),
                    5 => Ok(InfixOp::Eq),
                    6 => Ok(InfixOp::Gt),
                    7 => Ok(InfixOp::Lt),
                    8 => Ok(InfixOp::Ne),
                    9 => Ok(InfixOp::Ge),
                    10 => Ok(InfixOp::Le),
                    11 => Ok(InfixOp::BitwiseAnd),
                    12 => Ok(InfixOp::BitwiseOr),
                    13 => Ok(InfixOp::LogicalAnd),
                    14 => Ok(InfixOp::LogicalOr),
                    15 => Ok(InfixOp::Xor),
                    16 => Ok(InfixOp::ShiftRight),
                    17 => Ok(InfixOp::ShiftLeft),
                    18 => Ok(InfixOp::Index),
                    19 => Ok(InfixOp::Concat),
                    20 => Ok(InfixOp::Range),
                    21 => Ok(InfixOp::InclusiveRange),
                    22 => Ok(InfixOp::FieldModifier(InternedString::decode(buffer, index, session)?)),
                    23.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
