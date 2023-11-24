use super::{InfixOp, PostfixOp, PrefixOp};
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_intern::InternedString;

impl Endec for PrefixOp {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            PrefixOp::Not => { buf.push(0); },
            PrefixOp::Neg => { buf.push(1); },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, _: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(PrefixOp::Not),
                    1 => Ok(PrefixOp::Neg),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
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

    fn decode(buf: &[u8], ind: &mut usize, _: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(PostfixOp::Range),
                    1 => Ok(PostfixOp::QuestionMark),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
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
            InfixOp::Index => { buf.push(15); },
            InfixOp::Concat => { buf.push(16); },
            InfixOp::Append => { buf.push(17); },
            InfixOp::Prepend => { buf.push(18); },
            InfixOp::Range => { buf.push(19); },
            InfixOp::InclusiveRange => { buf.push(20); },
            InfixOp::FieldModifier(id) => {
                buf.push(21);
                id.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

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
                    15 => Ok(InfixOp::Index),
                    16 => Ok(InfixOp::Concat),
                    17 => Ok(InfixOp::Append),
                    18 => Ok(InfixOp::Prepend),
                    19 => Ok(InfixOp::Range),
                    20 => Ok(InfixOp::InclusiveRange),
                    21 => Ok(InfixOp::FieldModifier(InternedString::decode(buf, ind, session)?)),
                    22.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
