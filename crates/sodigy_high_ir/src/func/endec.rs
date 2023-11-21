use super::{Arg, Func, FuncDeco};
use crate::{Type, expr::Expr};
use sodigy_ast::{GenericDef, IdentWithSpan};
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

impl Endec for Func {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.args.encode(buf, session);
        self.generics.encode(buf, session);
        self.ret_val.encode(buf, session);
        self.ret_ty.encode(buf, session);
        self.decorators.encode(buf, session);
        self.doc.encode(buf, session);
        self.uid.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(Func {
            name: IdentWithSpan::decode(buf, ind, session)?,
            args: Option::<Vec<Arg>>::decode(buf, ind, session)?,
            generics: Vec::<GenericDef>::decode(buf, ind, session)?,
            ret_val: Expr::decode(buf, ind, session)?,
            ret_ty: Option::<Type>::decode(buf, ind, session)?,
            decorators: FuncDeco::decode(buf, ind, session)?,
            doc: Option::<InternedString>::decode(buf, ind, session)?,
            uid: Uid::decode(buf, ind, session)?,
        })
    }
}

impl Endec for Arg {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.ty.encode(buf, session);
        self.has_question_mark.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(Arg {
            name: IdentWithSpan::decode(buf, ind, session)?,
            ty: Option::<Type>::decode(buf, ind, session)?,
            has_question_mark: bool::decode(buf, ind, session)?,
        })
    }
}

impl Endec for FuncDeco {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        todo!()
    }
}
