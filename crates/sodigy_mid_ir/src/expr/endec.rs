use super::{Expr, ExprKind, MirFunc};
use crate::ty::Type;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};
use sodigy_intern::InternedNumeric;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

impl Endec for Expr {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.span.encode(buffer, session);
        self.ty.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Expr {
            kind: ExprKind::decode(buffer, index, session)?,
            span: SpanRange::decode(buffer, index, session)?,
            ty: Option::<Type>::decode(buffer, index, session)?,
        })
    }
}

impl Endec for ExprKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ExprKind::Integer(n) => {
                buffer.push(0);
                n.encode(buffer, session);
            },
            ExprKind::LocalValue { origin, key } => {
                buffer.push(1);
                origin.encode(buffer, session);
                key.encode(buffer, session);
            },
            ExprKind::Object(uid) => {
                buffer.push(2);
                uid.encode(buffer, session);
            },
            ExprKind::Call { func, args, tail_call } => {
                buffer.push(3);
                func.encode(buffer, session);
                args.encode(buffer, session);
                tail_call.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ExprKind::Integer(InternedNumeric::decode(buffer, index, session)?)),
                    1 => Ok(ExprKind::LocalValue {
                        origin: Uid::decode(buffer, index, session)?,
                        key: u32::decode(buffer, index, session)?,
                    }),
                    2 => Ok(ExprKind::Object(Uid::decode(buffer, index, session)?)),
                    3 => Ok(ExprKind::Call {
                        func: MirFunc::decode(buffer, index, session)?,
                        args: Vec::<Expr>::decode(buffer, index, session)?,
                        tail_call: bool::decode(buffer, index, session)?,
                    }),
                    4.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for MirFunc {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            MirFunc::Static(uid) => {
                buffer.push(0);
                uid.encode(buffer, session);
            },
            MirFunc::Dynamic(expr) => {
                buffer.push(1);
                expr.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(MirFunc::Static(Uid::decode(buffer, index, session)?)),
                    1 => Ok(MirFunc::Dynamic(Box::<Expr>::decode(buffer, index, session)?)),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
