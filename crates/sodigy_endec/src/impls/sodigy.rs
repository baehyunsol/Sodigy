// modules that cannot depend on `sodigy_endec`

use crate::{Endec, EndecError, EndecSession};
use crate::session::EncodedInternal;
use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_keyword::Keyword;
use sodigy_number::{BigNumber, SodigyNumber};

impl Endec for InternedString {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        // TODO: optimization: if this InternedString appears only once, don't intern it: just encode the raw string!
        let e = session.encode_intern_str(*self);
        e.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let e = EncodedInternal::decode(buffer, index, session)?;
        Ok(session.decode_intern_str(e)?)
    }
}

impl Endec for InternedNumeric {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        // TODO: optimization: if this InternedNumeric appears only once, don't intern it: just encode the raw SodigyNumber!
        let e = session.encode_intern_num(*self);
        e.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let e = EncodedInternal::decode(buffer, index, session)?;
        Ok(session.decode_intern_num(e)?)
    }
}

impl Endec for Keyword {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            Keyword::Let => { buffer.push(0); },
            Keyword::Enum => { buffer.push(1); },
            Keyword::Struct => { buffer.push(2); },
            Keyword::Module => { buffer.push(3); },
            Keyword::Import => { buffer.push(4); },
            Keyword::As => { buffer.push(5); },
            Keyword::From => { buffer.push(6); },
            Keyword::In => { buffer.push(7); },
            Keyword::If => { buffer.push(8); },
            Keyword::Else => { buffer.push(9); },
            Keyword::Pattern => { buffer.push(10); },
            Keyword::Match => { buffer.push(11); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Keyword::Let),
                    1 => Ok(Keyword::Enum),
                    2 => Ok(Keyword::Struct),
                    3 => Ok(Keyword::Module),
                    4 => Ok(Keyword::Import),
                    5 => Ok(Keyword::As),
                    6 => Ok(Keyword::From),
                    7 => Ok(Keyword::In),
                    8 => Ok(Keyword::If),
                    9 => Ok(Keyword::Else),
                    10 => Ok(Keyword::Pattern),
                    11 => Ok(Keyword::Match),
                    12.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for SodigyNumber {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            SodigyNumber::Big(n) => {
                buffer.push(0);
                n.encode(buffer, session);
            },
            SodigyNumber::SmallInt(n) => {
                buffer.push(1);
                n.encode(buffer, session);
            },
            SodigyNumber::SmallRatio(n) => {
                buffer.push(2);
                n.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(SodigyNumber::Big(Box::new(BigNumber::decode(buffer, index, session)?))),
                    1 => Ok(SodigyNumber::SmallInt(u64::decode(buffer, index, session)?)),
                    2 => Ok(SodigyNumber::SmallRatio(u64::decode(buffer, index, session)?)),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for BigNumber {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.digits.encode(buffer, session);
        self.exp.encode(buffer, session);
        self.is_integer.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(BigNumber {
            digits: Vec::<u8>::decode(buffer, index, session)?,
            exp: i64::decode(buffer, index, session)?,
            is_integer: bool::decode(buffer, index, session)?,
        })
    }
}
