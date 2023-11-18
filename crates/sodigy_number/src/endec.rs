use crate::{BigNumber, SodigyNumber};
use sodigy_endec::{Endec, EndecErr};

impl Endec for SodigyNumber {
    fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            SodigyNumber::Big(n) => {
                buf.push(0);
                n.encode(buf);
            },
            SodigyNumber::SmallInt(n) => {
                buf.push(1);
                n.encode(buf);
            },
            SodigyNumber::SmallRatio(n) => {
                buf.push(2);
                n.encode(buf);
            },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(SodigyNumber::Big(Box::new(BigNumber::decode(buf, ind)?))),
                    1 => Ok(SodigyNumber::SmallInt(u64::decode(buf, ind)?)),
                    2 => Ok(SodigyNumber::SmallRatio(u64::decode(buf, ind)?)),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl Endec for BigNumber {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.digits.encode(buf);
        self.exp.encode(buf);
        self.is_integer.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(BigNumber {
            digits: Vec::<u8>::decode(buf, ind)?,
            exp: i64::decode(buf, ind)?,
            is_integer: bool::decode(buf, ind)?,
        })
    }
}
