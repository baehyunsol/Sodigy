use super::{Expr, ExprKind};
use sodigy_endec::{Endec, EndecErr};
use sodigy_span::SpanRange;

impl Endec for Expr {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.kind.encode(buf);
        self.span.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(Expr {
            kind: ExprKind::decode(buf, ind)?,
            span: SpanRange::decode(buf, ind)?,
        })
    }
}

impl Endec for ExprKind {
    fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            ExprKind::Identifier(id) => {
                buf.push(0);
                id.encode(buf);
            },
            ExprKind::Integer(n) => {
                buf.push(1);
                n.encode(buf);
            },
            ExprKind::Ratio(n) => {
                buf.push(2);
                n.encode(buf);
            },
            ExprKind::Char(c) => {
                buf.push(3);
                c.encode(buf);
            },
            _ => todo!(),
        }
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    _ => todo!(),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
