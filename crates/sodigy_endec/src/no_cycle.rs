// These are defined in this module in order to avoid cyclic dependencies.

use crate::{Endec, EndecError, EndecSession};
use crate::session::EncodedInternal;
use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_keyword::Keyword;
use sodigy_number::{BigNumber, SodigyNumber};

impl Endec for InternedString {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        // TODO: optimization: if this InternedString appears only once, don't intern it: just encode the raw string!
        let e = session.encode_intern_str(*self);
        e.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let e = EncodedInternal::decode(buf, index, session)?;
        Ok(session.decode_intern_str(e)?)
    }
}

impl Endec for InternedNumeric {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        // TODO: optimization: if this InternedNumeric appears only once, don't intern it: just encode the raw SodigyNumber!
        let e = session.encode_intern_num(*self);
        e.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let e = EncodedInternal::decode(buf, index, session)?;
        Ok(session.decode_intern_num(e)?)
    }
}

impl Endec for Keyword {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            Keyword::Let => { buf.push(0); },
            Keyword::Enum => { buf.push(1); },
            Keyword::Struct => { buf.push(2); },
            Keyword::Module => { buf.push(3); },
            Keyword::Import => { buf.push(4); },
            Keyword::As => { buf.push(5); },
            Keyword::From => { buf.push(6); },
            Keyword::If => { buf.push(7); },
            Keyword::Else => { buf.push(8); },
            Keyword::Pattern => { buf.push(9); },
            Keyword::Match => { buf.push(10); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
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
                    7 => Ok(Keyword::If),
                    8 => Ok(Keyword::Else),
                    9 => Ok(Keyword::Pattern),
                    10 => Ok(Keyword::Match),
                    11.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for SodigyNumber {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            SodigyNumber::Big(n) => {
                buf.push(0);
                n.encode(buf, session);
            },
            SodigyNumber::SmallInt(n) => {
                buf.push(1);
                n.encode(buf, session);
            },
            SodigyNumber::SmallRatio(n) => {
                buf.push(2);
                n.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(SodigyNumber::Big(Box::new(BigNumber::decode(buf, index, session)?))),
                    1 => Ok(SodigyNumber::SmallInt(u64::decode(buf, index, session)?)),
                    2 => Ok(SodigyNumber::SmallRatio(u64::decode(buf, index, session)?)),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for BigNumber {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.digits.encode(buf, session);
        self.exp.encode(buf, session);
        self.is_integer.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(BigNumber {
            digits: Vec::<u8>::decode(buf, index, session)?,
            exp: i64::decode(buf, index, session)?,
            is_integer: bool::decode(buf, index, session)?,
        })
    }
}
