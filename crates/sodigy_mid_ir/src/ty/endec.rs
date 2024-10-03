use super::Type;
use crate::expr::Expr;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};
use sodigy_uid::Uid;

impl Endec for Type {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            Type::HasToBeInferred => {
                buffer.push(0);
            },
            Type::HasToBeLowered(e) => {
                buffer.push(1);
                e.encode(buffer, session);
            },
            Type::Simple(u) => {
                buffer.push(2);
                u.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Type::HasToBeInferred),
                    1 => Ok(Type::HasToBeLowered(Box::<Expr>::decode(buffer, index, session)?)),
                    2 => Ok(Type::Simple(Uid::decode(buffer, index, session)?)),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
