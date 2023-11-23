use super::Punct;
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_intern::InternedString;

impl Endec for Punct {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            Punct::At => { buf.push(0); },
            Punct::Add => { buf.push(1); },
            Punct::Sub => { buf.push(2); },
            Punct::Mul => { buf.push(3); },
            Punct::Div => { buf.push(4); },
            Punct::Rem => { buf.push(5); },
            Punct::Not => { buf.push(6); },
            Punct::Concat => { buf.push(7); },
            Punct::Assign => { buf.push(8); },
            Punct::Eq => { buf.push(9); },
            Punct::Gt => { buf.push(10); },
            Punct::Lt => { buf.push(11); },
            Punct::Ne => { buf.push(12); },
            Punct::Ge => { buf.push(13); },
            Punct::Le => { buf.push(14); },
            Punct::And => { buf.push(15); },
            Punct::AndAnd => { buf.push(16); },
            Punct::Or => { buf.push(17); },
            Punct::OrOr => { buf.push(18); },
            Punct::Comma => { buf.push(19); },
            Punct::Dot => { buf.push(20); },
            Punct::Colon => { buf.push(21); },
            Punct::SemiColon => { buf.push(22); },
            Punct::DotDot => { buf.push(23); },
            Punct::Backslash => { buf.push(24); },
            Punct::Dollar => { buf.push(25); },
            Punct::Backtick => { buf.push(26); },
            Punct::QuestionMark => { buf.push(27); },
            Punct::InclusiveRange => { buf.push(28); },
            Punct::RArrow => { buf.push(29); },
            Punct::Append => { buf.push(30); },
            Punct::Prepend => { buf.push(31); },
            Punct::FieldModifier(id) => {
                buf.push(32);
                id.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

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
                    15 => Ok(Punct::And),
                    16 => Ok(Punct::AndAnd),
                    17 => Ok(Punct::Or),
                    18 => Ok(Punct::OrOr),
                    19 => Ok(Punct::Comma),
                    20 => Ok(Punct::Dot),
                    21 => Ok(Punct::Colon),
                    22 => Ok(Punct::SemiColon),
                    23 => Ok(Punct::DotDot),
                    24 => Ok(Punct::Backslash),
                    25 => Ok(Punct::Dollar),
                    26 => Ok(Punct::Backtick),
                    27 => Ok(Punct::QuestionMark),
                    28 => Ok(Punct::InclusiveRange),
                    29 => Ok(Punct::RArrow),
                    30 => Ok(Punct::Append),
                    31 => Ok(Punct::Prepend),
                    32 => Ok(Punct::FieldModifier(InternedString::decode(buf, ind, session)?)),
                    33.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
