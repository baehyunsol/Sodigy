use super::Punct;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;

impl Endec for Punct {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            Punct::At => { buffer.push(0); },
            Punct::Add => { buffer.push(1); },
            Punct::Sub => { buffer.push(2); },
            Punct::Mul => { buffer.push(3); },
            Punct::Div => { buffer.push(4); },
            Punct::Rem => { buffer.push(5); },
            Punct::Not => { buffer.push(6); },
            Punct::Concat => { buffer.push(7); },
            Punct::Assign => { buffer.push(8); },
            Punct::Eq => { buffer.push(9); },
            Punct::Gt => { buffer.push(10); },
            Punct::Lt => { buffer.push(11); },
            Punct::Ne => { buffer.push(12); },
            Punct::Ge => { buffer.push(13); },
            Punct::Le => { buffer.push(14); },
            Punct::GtGt => { buffer.push(15);}
            Punct::LtLt => { buffer.push(16);}
            Punct::And => { buffer.push(17); },
            Punct::AndAnd => { buffer.push(18); },
            Punct::Or => { buffer.push(19); },
            Punct::OrOr => { buffer.push(20); },
            Punct::Xor => { buffer.push(21); },
            Punct::Comma => { buffer.push(22); },
            Punct::Dot => { buffer.push(23); },
            Punct::Colon => { buffer.push(24); },
            Punct::SemiColon => { buffer.push(25); },
            Punct::DotDot => { buffer.push(26); },
            Punct::Backslash => { buffer.push(27); },
            Punct::Dollar => { buffer.push(28); },
            Punct::Backtick => { buffer.push(29); },
            Punct::QuestionMark => { buffer.push(30); },
            Punct::InclusiveRange => { buffer.push(31); },
            Punct::RArrow => { buffer.push(32); },
            Punct::FieldModifier(id) => {
                buffer.push(33);
                id.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Punct::At),
                    1 => Ok(Punct::Add),
                    2 => Ok(Punct::Sub),
                    3 => Ok(Punct::Mul),
                    4 => Ok(Punct::Div),
                    5 => Ok(Punct::Rem),
                    6 => Ok(Punct::Not),
                    7 => Ok(Punct::Concat),
                    8 => Ok(Punct::Assign),
                    9 => Ok(Punct::Eq),
                    10 => Ok(Punct::Gt),
                    11 => Ok(Punct::Lt),
                    12 => Ok(Punct::Ne),
                    13 => Ok(Punct::Ge),
                    14 => Ok(Punct::Le),
                    15 => Ok(Punct::GtGt),
                    16 => Ok(Punct::LtLt),
                    17 => Ok(Punct::And),
                    18 => Ok(Punct::AndAnd),
                    19 => Ok(Punct::Or),
                    20 => Ok(Punct::OrOr),
                    21 => Ok(Punct::Xor),
                    22 => Ok(Punct::Comma),
                    23 => Ok(Punct::Dot),
                    24 => Ok(Punct::Colon),
                    25 => Ok(Punct::SemiColon),
                    26 => Ok(Punct::DotDot),
                    27 => Ok(Punct::Backslash),
                    28 => Ok(Punct::Dollar),
                    29 => Ok(Punct::Backtick),
                    30 => Ok(Punct::QuestionMark),
                    31 => Ok(Punct::InclusiveRange),
                    32 => Ok(Punct::RArrow),
                    33 => Ok(Punct::FieldModifier(InternedString::decode(buffer, index, session)?)),
                    34.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
