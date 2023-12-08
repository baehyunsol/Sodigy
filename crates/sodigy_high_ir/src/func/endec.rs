use super::{Arg, Func, FuncDeco, FuncKind};
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
        self.return_val.encode(buf, session);
        self.return_ty.encode(buf, session);
        self.decorators.encode(buf, session);
        self.doc.encode(buf, session);
        self.kind.encode(buf, session);
        self.uid.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(Func {
            name: IdentWithSpan::decode(buf, ind, session)?,
            args: Option::<Vec<Arg>>::decode(buf, ind, session)?,
            generics: Vec::<GenericDef>::decode(buf, ind, session)?,
            return_val: Expr::decode(buf, ind, session)?,
            return_ty: Option::<Type>::decode(buf, ind, session)?,
            decorators: FuncDeco::decode(buf, ind, session)?,
            doc: Option::<InternedString>::decode(buf, ind, session)?,
            kind: FuncKind::decode(buf, ind, session)?,
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

impl Endec for FuncKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            FuncKind::Normal => { buf.push(0); },
            FuncKind::Lambda => { buf.push(1); },
            FuncKind::Enum { variants } => {
                buf.push(2);
                variants.encode(buf, session);
            },
            FuncKind::EnumVariant { parent } => {
                buf.push(3);
                parent.encode(buf, session);
            },
            FuncKind::StructConstr => { buf.push(4); },
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(FuncKind::Normal),
                    1 => Ok(FuncKind::Lambda),
                    2 => Ok(FuncKind::Enum {
                        variants: Vec::<Uid>::decode(buf, ind, session)?,
                    }),
                    3 => Ok(FuncKind::EnumVariant {
                        parent: Uid::decode(buf, ind, session)?,
                    }),
                    4 => Ok(FuncKind::StructConstr),
                    5.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
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
