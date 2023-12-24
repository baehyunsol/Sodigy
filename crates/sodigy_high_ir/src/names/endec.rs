use super::{IdentWithOrigin, NameOrigin};
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

impl Endec for IdentWithOrigin {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buf, session);
        self.1.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(IdentWithOrigin(
            InternedString::decode(buf, index, session)?,
            NameOrigin::decode(buf, index, session)?,
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

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(NameOrigin::Prelude),
                    1 => Ok(NameOrigin::FuncArg { index: usize::decode(buf, index, session)? }),
                    2 => Ok(NameOrigin::FuncGeneric { index: usize::decode(buf, index, session)? }),
                    3 => Ok(NameOrigin::Local { origin: Uid::decode(buf, index, session)? }),
                    4 => Ok(NameOrigin::Global { origin: Option::<Uid>::decode(buf, index, session)? }),
                    5 => Ok(NameOrigin::Captured {
                        lambda: Uid::decode(buf, index, session)?,
                        index: usize::decode(buf, index, session)?,
                    }),
                    6.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
