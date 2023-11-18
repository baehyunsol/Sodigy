use super::{Arg, Func, FuncDeco};
use crate::{Type, expr::Expr};
use sodigy_ast::{GenericDef, IdentWithSpan};
use sodigy_endec::{Endec, EndecErr};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

impl Endec for Func {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.name.encode(buf);
        self.args.encode(buf);
        self.generics.encode(buf);
        self.ret_val.encode(buf);
        self.ret_ty.encode(buf);
        self.decorators.encode(buf);
        self.doc.encode(buf);
        self.uid.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(Func {
            name: IdentWithSpan::decode(buf, ind)?,
            args: Option::<Vec<Arg>>::decode(buf, ind)?,
            generics: Vec::<GenericDef>::decode(buf, ind)?,
            ret_val: Expr::decode(buf, ind)?,
            ret_ty: Option::<Type>::decode(buf, ind)?,
            decorators: FuncDeco::decode(buf, ind)?,
            doc: Option::<InternedString>::decode(buf, ind)?,
            uid: Uid::decode(buf, ind)?,
        })
    }
}

impl Endec for Arg {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.name.encode(buf);
        self.ty.encode(buf);
        self.has_question_mark.encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        Ok(Arg {
            name: IdentWithSpan::decode(buf, ind)?,
            ty: Option::<Type>::decode(buf, ind)?,
            has_question_mark: bool::decode(buf, ind)?,
        })
    }
}

impl Endec for FuncDeco {
    fn encode(&self, buf: &mut Vec<u8>) {
        todo!()
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        todo!()
    }
}
