use super::{IdentWithOrigin, NameOrigin};
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

impl Endec for IdentWithOrigin {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
        self.1.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(IdentWithOrigin(
            InternedString::decode(buf, ind, session)?,
            NameOrigin::decode(buf, ind, session)?,
        ))
    }
}

impl Endec for NameOrigin {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            NameOrigin::Prelude => {
                buf.push(0);
            },
            NameOrigin::FuncArg { index } => {
                buf.push(1);
                index.encode(buf, session);
            },
            NameOrigin::FuncGeneric { index } => {
                buf.push(2);
                index.encode(buf, session);
            },
            NameOrigin::Local { origin } => {
                buf.push(3);
                origin.encode(buf, session);
            },
            NameOrigin::Global { origin } => {
                buf.push(4);
                origin.encode(buf, session);
            },
            NameOrigin::Captured { lambda, index } => {
                buf.push(5);
                lambda.encode(buf, session);
                index.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(NameOrigin::Prelude),
                    1 => Ok(NameOrigin::FuncArg { index: usize::decode(buf, ind, session)? }),
                    2 => Ok(NameOrigin::FuncGeneric { index: usize::decode(buf, ind, session)? }),
                    3 => Ok(NameOrigin::Local { origin: Uid::decode(buf, ind, session)? }),
                    4 => Ok(NameOrigin::Global { origin: Option::<Uid>::decode(buf, ind, session)? }),
                    5 => Ok(NameOrigin::Captured {
                        lambda: Uid::decode(buf, ind, session)?,
                        index: usize::decode(buf, ind, session)?,
                    }),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
