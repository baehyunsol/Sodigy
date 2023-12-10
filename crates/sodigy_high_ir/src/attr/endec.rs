use super::{Attribute, Decorator};
use crate::expr::Expr;
use sodigy_endec::{Endec, EndecErr, EndecSession};
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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(Attribute::DocComment(IdentWithSpan::decode(buf, ind, session)?)),
                    1 => Ok(Attribute::Decorator(Decorator::decode(buf, ind, session)?)),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl Endec for Decorator {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.args.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(Decorator {
            name: Vec::<IdentWithSpan>::decode(buf, ind, session)?,
            args: Option::<Vec<Expr>>::decode(buf, ind, session)?,
        })
    }
}
