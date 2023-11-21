// These are defined in this module in order to avoid cyclic dependencies.

use crate::{Endec, EndecErr, EndecSession};
use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_keyword::Keyword;
use sodigy_number::{BigNumber, SodigyNumber};

impl Endec for InternedString {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        let e = session.encode_intern_str(*self);
        e.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        todo!()
    }
}

impl Endec for InternedNumeric {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        let e = session.encode_intern_num(*self);
        e.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        todo!()
    }
}

impl Endec for Keyword {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        todo!()
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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(SodigyNumber::Big(Box::new(BigNumber::decode(buf, ind, session)?))),
                    1 => Ok(SodigyNumber::SmallInt(u64::decode(buf, ind, session)?)),
                    2 => Ok(SodigyNumber::SmallRatio(u64::decode(buf, ind, session)?)),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl Endec for BigNumber {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.digits.encode(buf, session);
        self.exp.encode(buf, session);
        self.is_integer.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(BigNumber {
            digits: Vec::<u8>::decode(buf, ind, session)?,
            exp: i64::decode(buf, ind, session)?,
            is_integer: bool::decode(buf, ind, session)?,
        })
    }
}
