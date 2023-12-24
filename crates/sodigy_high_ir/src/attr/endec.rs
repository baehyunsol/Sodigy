use super::{Attribute, Decorator};
use crate::expr::Expr;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_ast::IdentWithSpan;

impl Endec for Attribute {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            Attribute::DocComment(d) => {
                buf.push(0);
                d.encode(buf, session);
            },
            Attribute::Decorator(d) => {
                buf.push(1);
                d.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Attribute::DocComment(IdentWithSpan::decode(buf, index, session)?)),
                    1 => Ok(Attribute::Decorator(Decorator::decode(buf, index, session)?)),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for Decorator {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.args.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Decorator {
            name: Vec::<IdentWithSpan>::decode(buf, index, session)?,
            args: Option::<Vec<Expr>>::decode(buf, index, session)?,
        })
    }
}
