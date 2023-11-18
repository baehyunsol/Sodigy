use super::{IdentWithOrigin, NameOrigin};
use sodigy_endec::{Endec, EndecErr};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

impl Endec for IdentWithOrigin {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.0.encode(buf);
        self.1.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(IdentWithOrigin(
            InternedString::decode(buf, ind)?,
            NameOrigin::decode(buf, ind)?,
        ))
    }
}

impl Endec for NameOrigin {
    fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            NameOrigin::Prelude => {
                buf.push(0);
            },
            NameOrigin::FuncArg { index } => {
                buf.push(1);
                index.encode(buf);
            },
            NameOrigin::FuncGeneric { index } => {
                buf.push(2);
                index.encode(buf);
            },
            NameOrigin::Local { origin } => {
                buf.push(3);
                origin.encode(buf);
            },
            NameOrigin::Global { origin } => {
                buf.push(4);
                origin.encode(buf);
            },
            NameOrigin::Captured { lambda, index } => {
                buf.push(5);
                lambda.encode(buf);
                index.encode(buf);
            },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(NameOrigin::Prelude),
                    1 => Ok(NameOrigin::FuncArg { index: usize::decode(buf, ind)? }),
                    2 => Ok(NameOrigin::FuncGeneric { index: usize::decode(buf, ind)? }),
                    3 => Ok(NameOrigin::Local { origin: Uid::decode(buf, ind)? }),
                    4 => Ok(NameOrigin::Global { origin: Option::<Uid>::decode(buf, ind)? }),
                    5 => Ok(NameOrigin::Captured {
                        lambda: Uid::decode(buf, ind)?,
                        index: usize::decode(buf, ind)?,
                    }),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
