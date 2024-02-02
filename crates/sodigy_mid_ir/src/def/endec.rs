use super::{Arg, Def};
use crate::expr::Expr;
use crate::ty::Type;
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_uid::Uid;

impl Endec for Def {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.args.encode(buf, session);
        self.return_ty.encode(buf, session);
        self.return_val.encode(buf, session);
        self.uid.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Def {
            name: IdentWithSpan::decode(buf, index, session)?,
            args: Option::<Vec<Arg>>::decode(buf, index, session)?,
            return_ty: Type::decode(buf, index, session)?,
            return_val: Expr::decode(buf, index, session)?,
            uid: Uid::decode(buf, index, session)?,
        })
    }
}

impl Endec for Arg {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.ty.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Arg {
            name: IdentWithSpan::decode(buf, index, session)?,
            ty: Type::decode(buf, index, session)?,
        })
    }
}
