use super::{Expr, ExprKind};
use crate::ty::Type;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_span::SpanRange;

impl Endec for Expr {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.ty.encode(buffer, session);
        self.span.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Expr {
            kind: ExprKind::decode(buffer, index, session)?,
            ty: Type::decode(buffer, index, session)?,
            span: SpanRange::decode(buffer, index, session)?,
        })
    }
}

impl Endec for ExprKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}
