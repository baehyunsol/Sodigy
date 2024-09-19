use super::{
    Func,
    LocalValue,
    LocalValueGraph, 
    LocalValueKey,
    LocalValueRef,
    MaybeInit,
};
use crate::expr::Expr;
use crate::ty::Type;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};
use sodigy_high_ir::{self as hir, NameBindingType};
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;
use std::collections::HashMap;

impl Endec for Func {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.return_type.encode(buffer, session);
        self.return_value.encode(buffer, session);
        self.local_values.encode(buffer, session);
        self.uid.encode(buffer, session);
        self.local_values_reachable_from_return_value.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Func {
            name: IdentWithSpan::decode(buffer, index, session)?,
            return_type: Type::decode(buffer, index, session)?,
            return_value: Expr::decode(buffer, index, session)?,
            local_values: HashMap::<u32, LocalValue>::decode(buffer, index, session)?,
            uid: Uid::decode(buffer, index, session)?,
            local_values_reachable_from_return_value: HashMap::<LocalValueKey, LocalValueRef>::decode(buffer, index, session)?,
        })
    }
}

impl Endec for LocalValue {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.value.encode(buffer, session);
        self.ty.encode(buffer, session);
        self.parent_func.encode(buffer, session);
        self.parent_scope.encode(buffer, session);
        self.name_binding_type.encode(buffer, session);
        self.is_real.encode(buffer, session);
        self.key.encode(buffer, session);
        self.graph.encode(buffer, session);
        self.is_valid.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(LocalValue {
            name: IdentWithSpan::decode(buffer, index, session)?,
            value: MaybeInit::<hir::Expr, Expr>::decode(buffer, index, session)?,
            ty: MaybeInit::<hir::Type, Type>::decode(buffer, index, session)?,
            parent_func: Uid::decode(buffer, index, session)?,
            parent_scope: Option::<Uid>::decode(buffer, index, session)?,
            name_binding_type: NameBindingType::decode(buffer, index, session)?,
            is_real: bool::decode(buffer, index, session)?,
            key: u32::decode(buffer, index, session)?,
            graph: Option::<LocalValueGraph>::decode(buffer, index, session)?,
            is_valid: bool::decode(buffer, index, session)?,
        })
    }
}

impl<T: Endec, U: Endec> Endec for MaybeInit<T, U> {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            MaybeInit::None => {
                buffer.push(0);
            },
            MaybeInit::Uninit(v) => {
                buffer.push(1);
                v.encode(buffer, session);
            },
            MaybeInit::Init(v) => {
                buffer.push(2);
                v.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(MaybeInit::None),
                    1 => Ok(MaybeInit::Uninit(T::decode(buffer, index, session)?)),
                    2 => Ok(MaybeInit::Init(U::decode(buffer, index, session)?)),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for LocalValueGraph {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}

impl Endec for LocalValueRef {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.must.encode(buffer, session);
        self.cond.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(LocalValueRef {
            must: u32::decode(buffer, index, session)?,
            cond: u32::decode(buffer, index, session)?,
        })
    }
}
