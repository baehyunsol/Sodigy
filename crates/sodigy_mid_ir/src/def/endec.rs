use super::{Arg, Def};
use crate::expr::Expr;
use crate::ty::Type;
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_uid::Uid;

impl Endec for Def {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.args.encode(buffer, session);
        self.return_ty.encode(buffer, session);
        self.return_val.encode(buffer, session);
        self.uid.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Def {
            name: IdentWithSpan::decode(buffer, index, session)?,
            args: Option::<Vec<Arg>>::decode(buffer, index, session)?,
            return_ty: Type::decode(buffer, index, session)?,
            return_val: Expr::decode(buffer, index, session)?,
            uid: Uid::decode(buffer, index, session)?,
        })
    }
}

impl Endec for Arg {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.ty.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Arg {
            name: IdentWithSpan::decode(buffer, index, session)?,
            ty: Type::decode(buffer, index, session)?,
        })
    }
}
