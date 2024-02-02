use super::{Expr, ExprKind};
use crate::ty::Type;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_span::SpanRange;

impl Endec for Expr {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.ty.encode(buf, session);
        self.span.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Expr {
            kind: ExprKind::decode(buf, index, session)?,
            ty: Type::decode(buf, index, session)?,
            span: SpanRange::decode(buf, index, session)?,
        })
    }
}

impl Endec for ExprKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}
