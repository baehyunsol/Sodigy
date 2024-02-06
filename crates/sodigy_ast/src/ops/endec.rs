use super::{InfixOp, PostfixOp, PrefixOp};
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;

impl Endec for PrefixOp {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            PrefixOp::Not => { buf.push(0); },
            PrefixOp::Neg => { buf.push(1); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
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
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            PostfixOp::Range => { buf.push(0); },
            PostfixOp::QuestionMark => { buf.push(1); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
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
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            InfixOp::Add => { buf.push(0); },
            InfixOp::Sub => { buf.push(1); },
            InfixOp::Mul => { buf.push(2); },
            InfixOp::Div => { buf.push(3); },
            InfixOp::Rem => { buf.push(4); },
            InfixOp::Eq => { buf.push(5); },
            InfixOp::Gt => { buf.push(6); },
            InfixOp::Lt => { buf.push(7); },
            InfixOp::Ne => { buf.push(8); },
            InfixOp::Ge => { buf.push(9); },
            InfixOp::Le => { buf.push(10); },
            InfixOp::BitwiseAnd => { buf.push(11); },
            InfixOp::BitwiseOr => { buf.push(12); },
            InfixOp::LogicalAnd => { buf.push(13); },
            InfixOp::LogicalOr => { buf.push(14); },
            InfixOp::Xor => { buf.push(15); },
            InfixOp::ShiftRight => { buf.push(16); },
            InfixOp::ShiftLeft => { buf.push(17); },
            InfixOp::As => { buf.push(18); },
            InfixOp::In => { buf.push(19); },
            InfixOp::Index => { buf.push(20); },
            InfixOp::Concat => { buf.push(21); },
            InfixOp::Range => { buf.push(22); },
            InfixOp::InclusiveRange => { buf.push(23); },
            InfixOp::FieldModifier(id) => {
                buf.push(24);
                id.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
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
                    18 => Ok(InfixOp::As),
                    19 => Ok(InfixOp::In),
                    20 => Ok(InfixOp::Index),
                    21 => Ok(InfixOp::Concat),
                    22 => Ok(InfixOp::Range),
                    23 => Ok(InfixOp::InclusiveRange),
                    24 => Ok(InfixOp::FieldModifier(InternedString::decode(buf, index, session)?)),
                    25.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
